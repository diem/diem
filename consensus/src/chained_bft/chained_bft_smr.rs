// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockStore,
        epoch_manager::EpochManager,
        event_processor::EventProcessor,
        network::{NetworkReceivers, NetworkTask},
        persistent_storage::{PersistentStorage, RecoveryData},
    },
    counters,
    state_replication::{StateComputer, StateMachineReplication, TxnManager},
    util::time_service::ClockTimeService,
};
use anyhow::Result;
use channel;
use consensus_types::common::{Author, Payload, Round};
use futures::{select, stream::StreamExt};
use libra_config::config::{ConsensusConfig, NodeConfig};
use libra_logger::prelude::*;
use libra_types::crypto_proxies::EpochInfo;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
use safety_rules::SafetyRulesManager;
use std::{sync::Arc, sync::RwLock, time::Instant};
use tokio::runtime::{Handle, Runtime};

/// All these structures need to be moved into EpochManager. Rather than make each one an option
/// and perform ugly unwraps, they are bundled here.
pub struct ChainedBftSMRInput<T> {
    network_sender: ConsensusNetworkSender,
    network_events: ConsensusNetworkEvents,
    safety_rules_manager: SafetyRulesManager<T>,
    config: ConsensusConfig,
    initial_data: RecoveryData<T>,
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
        author: Author,
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        node_config: &mut NodeConfig,
        runtime: Runtime,
        storage: Arc<dyn PersistentStorage<T>>,
        initial_data: RecoveryData<T>,
    ) -> Self {
        let input = ChainedBftSMRInput {
            network_sender,
            network_events,
            safety_rules_manager: SafetyRulesManager::new(node_config),
            config: node_config.consensus.clone(),
            initial_data,
        };

        Self {
            author,
            runtime: Some(runtime),
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
        mut event_processor: EventProcessor<T>,
        mut pacemaker_timeout_sender_rx: channel::Receiver<Round>,
        network_task: NetworkTask<T>,
        mut network_receivers: NetworkReceivers<T>,
    ) {
        let fut = async move {
            event_processor.start().await;
            loop {
                let pre_select_instant = Instant::now();
                let idle_duration;
                select! {
                    proposal_msg = network_receivers.proposals.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        event_processor.process_proposal_msg(proposal_msg).await;
                    }
                    block_retrieval = network_receivers.block_retrieval.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        event_processor.process_block_retrieval(block_retrieval).await;
                    }
                    vote_msg = network_receivers.votes.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        event_processor.process_vote(vote_msg).await;
                    }
                    local_timeout_round = pacemaker_timeout_sender_rx.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        event_processor.process_local_timeout(local_timeout_round).await;
                    }
                    sync_info_msg = network_receivers.sync_info_msgs.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        event_processor.process_sync_info_msg(sync_info_msg.0, sync_info_msg.1).await;
                    }
                    ledger_info = network_receivers.epoch_change.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        if epoch_manager.epoch() <= ledger_info.ledger_info().epoch() {
                            event_processor = epoch_manager.start_new_epoch(ledger_info).await;
                            // clean up all the previous messages from the old epochs
                            network_receivers.clear_prev_epoch_msgs();
                            event_processor.start().await;
                        }
                    }
                    different_epoch_and_peer = network_receivers.different_epoch.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        epoch_manager.process_different_epoch(different_epoch_and_peer.0, different_epoch_and_peer.1).await
                    }
                    epoch_retrieval_and_peer = network_receivers.epoch_retrieval.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        epoch_manager.process_epoch_retrieval(epoch_retrieval_and_peer.0, epoch_retrieval_and_peer.1).await
                    }
                }
                counters::EVENT_PROCESSING_LOOP_BUSY_DURATION_S
                    .observe_duration(pre_select_instant.elapsed() - idle_duration);
                counters::EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
            }
        };
        executor.spawn(network_task.start());
        executor.spawn(fut);
    }
}

impl<T: Payload> StateMachineReplication for ChainedBftSMR<T> {
    type Payload = T;

    /// We're following the steps to start
    /// 1. Construct the EpochManager from the latest libradb state
    /// 2. Construct per-epoch component with the fixed Validators provided by EpochManager including
    /// ProposerElection, Pacemaker, SafetyRules, Network(Populate with known validators), EventProcessor
    fn start(
        &mut self,
        txn_manager: Box<dyn TxnManager<Payload = Self::Payload>>,
        state_computer: Arc<dyn StateComputer<Payload = Self::Payload>>,
    ) -> Result<()> {
        let input = self.input.take().expect("already started, input is None");
        let executor = self
            .runtime
            .as_mut()
            .expect("Consensus start: No valid runtime found!")
            .handle();
        let time_service = Arc::new(ClockTimeService::new(executor.clone()));

        let (timeout_sender, timeout_receiver) =
            channel::new(1_024, &counters::PENDING_PACEMAKER_TIMEOUTS);
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        let epoch_info = Arc::new(RwLock::new(EpochInfo {
            epoch: input.initial_data.epoch(),
            verifier: input.initial_data.validators(),
        }));

        let mut epoch_mgr = EpochManager::new(
            self.author,
            Arc::clone(&epoch_info),
            input.config,
            time_service,
            self_sender,
            input.network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            self.storage.clone(),
            input.safety_rules_manager,
        );

        // Step 2
        let event_processor = epoch_mgr.start_epoch(input.initial_data);

        // TODO: this is test only, we should remove this
        self.block_store = Some(event_processor.block_store());

        let (network_task, network_receiver) =
            NetworkTask::new(epoch_info, input.network_events, self_receiver);

        Self::start_event_processing(
            executor.clone(),
            epoch_mgr,
            event_processor,
            timeout_receiver,
            network_task,
            network_receiver,
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
