// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockStore,
        epoch_manager::{EpochCheck, EpochManager, EpochMsg},
        event_processor::EventProcessor,
        network::{ConsensusDataRequest, FromNetworkMsg, NetworkTask},
        persistent_storage::PersistentStorage,
    },
    consensus_provider::ConsensusProvider,
    counters,
    state_replication::{StateComputer, TxnManager},
    util::time_service::ClockTimeService,
};
use anyhow::Result;
use channel;
use consensus_types::common::{Author, Payload, Round};
use futures::{channel::mpsc, select, stream::select, stream::StreamExt, Stream, TryStreamExt};
use libra_config::config::{ConsensusConfig, NodeConfig};
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::crypto_proxies::EpochInfo;
use network::proto::ConsensusMsg;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event};
use safety_rules::SafetyRulesManager;
use std::{sync::Arc, time::Instant};
use tokio::runtime::{self, Handle, Runtime};

/// All these structures need to be moved into EpochManager. Rather than make each one an option
/// and perform ugly unwraps, they are bundled here.
pub struct ChainedBftSMRInput<T> {
    network_sender: ConsensusNetworkSender,
    network_events: ConsensusNetworkEvents,
    safety_rules_manager: SafetyRulesManager<T>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    txn_manager: Box<dyn TxnManager<Payload = T>>,
    config: ConsensusConfig,
}

/// ChainedBFTSMR is the one to generate the components (BlockStore, Proposer, etc.) and start the
/// driver. ChainedBftSMR implements the StateMachineReplication, it is going to be used by
/// ConsensusProvider for the e2e flow.
pub struct ChainedBftSMR<T> {
    author: Author,
    runtime: Option<Runtime>,
    block_store: Option<Arc<BlockStore<T>>>,
    storage: Arc<dyn PersistentStorage<T>>,
    input: Option<ChainedBftSMRInput<T>>,
}

impl<T: Payload> ChainedBftSMR<T> {
    pub fn new(
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        node_config: &mut NodeConfig,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        storage: Arc<dyn PersistentStorage<T>>,
        txn_manager: Box<dyn TxnManager<Payload = T>>,
    ) -> Self {
        let input = ChainedBftSMRInput {
            network_sender,
            network_events,
            safety_rules_manager: SafetyRulesManager::new(node_config),
            state_computer,
            txn_manager,
            config: node_config.consensus.clone(),
        };

        Self {
            author: node_config.validator_network.as_ref().unwrap().peer_id,
            runtime: None,
            block_store: None,
            storage,
            input: Some(input),
        }
    }

    #[cfg(test)]
    pub fn author(&self) -> Author {
        self.author
    }

    #[cfg(test)]
    pub fn block_store(&self) -> Option<Arc<BlockStore<T>>> {
        self.block_store.clone()
    }

    fn start_event_processing(
        executor: Handle,
        mut epoch_manager: EpochManager<T>,
        mut event_processor: Box<EventProcessor<T>>,
        mut pacemaker_timeout_sender_rx: channel::Receiver<Round>,
        network_task: NetworkTask<T>,
        all_events: Box<dyn Stream<Item = anyhow::Result<Event<ConsensusMsg>>> + Send + Unpin>,
        data_request_sender: mpsc::UnboundedSender<ConsensusDataRequest>,
        mut data_receiver: mpsc::UnboundedReceiver<(AccountAddress, FromNetworkMsg<T>)>,
    ) {
        let fut = async move {
            event_processor.start().await;

            // Ask for new data on start-up.
            let _ = data_request_sender.unbounded_send(ConsensusDataRequest::NewData);

            loop {
                let pre_select_instant = Instant::now();
                let idle_duration;

                select! {
                    (peer_id, consensus_msg) = data_receiver.select_next_some() => {

                        idle_duration = pre_select_instant.elapsed();

                        let current_epoch_msg = match epoch_manager.check_epoch(&peer_id, consensus_msg).await {
                            Ok(EpochCheck::Current(msg)) => {
                                // Since the epoch did not change,
                                // and further processing of this EpochMsg by the event-processor
                                // cannot influence this,
                                // we can optimistically request
                                // another message from network.
                                let _ = data_request_sender.unbounded_send(ConsensusDataRequest::NewData);
                                msg
                            },
                            Ok(EpochCheck::NotCurrent) => {
                                // No further processing required of an EpochMsg,
                                // so we just request more data and go back to the select above.
                                let _ = data_request_sender.unbounded_send(ConsensusDataRequest::NewData);
                                continue;
                            },
                            Ok(EpochCheck::NewStart(processor)) => {
                                // The epoch changed:

                                // 1. Update the processor of epoch messages.
                                event_processor = processor;

                                // 2. Ask network to clear its buffer of epoch-specific messages.
                                let _ = data_request_sender.unbounded_send(ConsensusDataRequest::ClearEpochData);

                                // 3. Ask for more data.
                                let _ = data_request_sender.unbounded_send(ConsensusDataRequest::NewData);

                                // 4. Start the processor for the new epoch.
                                event_processor.start().await;
                                continue;
                            },
                            Err(e) => {
                                warn!("Failed to process network message {:?}", e);

                                // Even though the last message resulted in an error,
                                // we still want to get more data to process,
                                // request data and go to the select.
                                let _ = data_request_sender.unbounded_send(ConsensusDataRequest::NewData);
                                continue
                            },
                        };

                        match current_epoch_msg {
                            EpochMsg::Proposal(proposal) => {
                                event_processor.process_proposal_msg(proposal).await;
                            },
                            EpochMsg::RequestBlock(request) => {
                                event_processor.process_block_retrieval(request).await;
                            },
                            EpochMsg::Vote(vote_msg) => {
                                event_processor.process_vote(vote_msg).await;
                            },
                            EpochMsg::Sync(sync_info) => {
                                event_processor.process_sync_info_msg(sync_info, peer_id).await;
                            },
                        }
                    },
                    local_timeout_round = pacemaker_timeout_sender_rx.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        event_processor.process_local_timeout(local_timeout_round).await;
                    }
                }

                counters::EVENT_PROCESSING_LOOP_BUSY_DURATION_S
                    .observe_duration(pre_select_instant.elapsed() - idle_duration);
                counters::EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
            }
        };
        executor.spawn(network_task.start(all_events));
        executor.spawn(fut);
    }
}

impl<T: Payload> ConsensusProvider for ChainedBftSMR<T> {
    /// We're following the steps to start
    /// 1. Construct the EpochManager from the latest libradb state
    /// 2. Construct per-epoch component with the fixed Validators provided by EpochManager including
    /// ProposerElection, Pacemaker, SafetyRules, Network(Populate with known validators), EventProcessor
    fn start(&mut self) -> Result<()> {
        let mut runtime = runtime::Builder::new()
            .thread_name("consensus-")
            // Question: How many, if more than one at all, threads, does this runtime need?
            //
            // I find it somewhat hard to reason about opportunities for parallelism here.
            // The EpochManager and EventProcessor own a buch of other things, like the StorageWriteProxy,
            // but all calls into those "other things" are essentially sync, right?
            // So at the end of the day, we have only two tasks, NetworkTask and then the other "consensus" task.
            // So maybe two threads is enough, maybe even only one?
            //
            // The only opportunities for parallelism between those to tasks seem to be about
            // having NetworkTask buffer/drop incoming messages while the "consensus" task is processing a message.
            //
            // Are there any other opportunities, or need, for parallelism?
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime!");

        let initial_data = runtime.block_on(self.storage.start());
        let input = self.input.take().expect("already started, input is None");

        let executor = runtime.handle().clone();
        let time_service = Arc::new(ClockTimeService::new(executor.clone()));
        self.runtime = Some(runtime);

        let (timeout_sender, timeout_receiver) =
            channel::new(1_024, &counters::PENDING_PACEMAKER_TIMEOUTS);
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        let epoch_info = EpochInfo {
            epoch: initial_data.epoch(),
            verifier: initial_data.validators(),
        };

        let mut epoch_mgr = EpochManager::new(
            self.author,
            epoch_info,
            input.config,
            time_service,
            self_sender,
            input.network_sender,
            timeout_sender,
            input.txn_manager,
            input.state_computer,
            self.storage.clone(),
            input.safety_rules_manager,
        );

        // Step 2
        let event_processor = epoch_mgr.start_epoch(initial_data);

        // TODO: this is test only, we should remove this
        self.block_store = Some(event_processor.block_store());

        let (network_data_request_sender, network_data_request_receiver) = mpsc::unbounded();
        let (network_data_sender, network_data_receiver) = mpsc::unbounded();

        let network_task = NetworkTask::new(network_data_request_receiver, network_data_sender);

        let network_events = input.network_events.map_err(Into::<anyhow::Error>::into);
        let all_events = Box::new(select(network_events, self_receiver));

        Self::start_event_processing(
            executor,
            epoch_mgr,
            event_processor,
            timeout_receiver,
            network_task,
            all_events,
            network_data_request_sender,
            network_data_receiver,
        );

        debug!("Chained BFT SMR started.");
        Ok(())
    }

    /// Stop is synchronous: waits for all the worker threads to terminate.
    fn stop(&mut self) {
        if let Some(_rt) = self.runtime.take() {
            debug!("Chained BFT SMR stopped.")
        }
    }
}
