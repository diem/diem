// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockStore,
        epoch_manager::EpochManager,
        network::{NetworkReceivers, NetworkTask},
        network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
        persistent_liveness_storage::PersistentLivenessStorage,
    },
    consensus_provider::ConsensusProvider,
    counters,
    state_replication::{StateComputer, TxnManager},
    util::time_service::ClockTimeService,
};
use anyhow::Result;
use consensus_types::common::{Author, Payload, Round};
use futures::{select, stream::StreamExt};
use libra_config::config::{ConsensusConfig, NodeConfig};
use libra_logger::prelude::*;
use safety_rules::SafetyRulesManager;
use std::{sync::Arc, time::Instant};
use tokio::runtime::{self, Handle, Runtime};

/// All these structures need to be moved into EpochManager. Rather than make each one an option
/// and perform ugly unwraps, they are bundled here.
pub struct ChainedBftSMRInput<T> {
    network_sender: ConsensusNetworkSender<T>,
    network_events: ConsensusNetworkEvents<T>,
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
    storage: Arc<dyn PersistentLivenessStorage<T>>,
    input: Option<ChainedBftSMRInput<T>>,
}

impl<T: Payload> ChainedBftSMR<T> {
    pub fn new(
        network_sender: ConsensusNetworkSender<T>,
        network_events: ConsensusNetworkEvents<T>,
        node_config: &mut NodeConfig,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        storage: Arc<dyn PersistentLivenessStorage<T>>,
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
        mut pacemaker_timeout_sender_rx: channel::Receiver<Round>,
        network_task: NetworkTask<T>,
        mut network_receivers: NetworkReceivers<T>,
    ) {
        let fut = async move {
            loop {
                let pre_select_instant = Instant::now();
                let idle_duration;
                select! {
                    msg = network_receivers.consensus_messages.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        epoch_manager.process_message(msg.0, msg.1)
                    }
                    block_retrieval = network_receivers.block_retrieval.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        epoch_manager.process_block_retrieval(block_retrieval)
                    }
                    round = pacemaker_timeout_sender_rx.select_next_some() => {
                        idle_duration = pre_select_instant.elapsed();
                        epoch_manager.process_local_timeout(round)
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

impl<T: Payload> ConsensusProvider for ChainedBftSMR<T> {
    /// We're following the steps to start
    /// 1. Construct the EpochManager from the latest libradb state
    /// 2. Construct per-epoch component with the fixed Validators provided by EpochManager including
    /// ProposerElection, Pacemaker, SafetyRules, Network(Populate with known validators), EventProcessor
    fn start(&mut self) -> Result<()> {
        let runtime = runtime::Builder::new()
            .thread_name("consensus-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime!");
        let input = self.input.take().expect("already started, input is None");

        let executor = runtime.handle().clone();
        let time_service = Arc::new(ClockTimeService::new());

        let (timeout_sender, timeout_receiver) =
            channel::new(1_024, &counters::PENDING_PACEMAKER_TIMEOUTS);
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);

        let mut epoch_mgr = EpochManager::new(
            self.author,
            input.config,
            time_service,
            self_sender,
            input.network_sender.clone(),
            timeout_sender,
            input.txn_manager,
            input.state_computer,
            self.storage.clone(),
            input.safety_rules_manager,
        );

        let (network_task, network_receiver) =
            NetworkTask::new(input.network_events, self_receiver);

        epoch_mgr.start_processor();

        // TODO: this is for testing, remove
        self.block_store = epoch_mgr.block_store();

        Self::start_event_processing(
            executor,
            epoch_mgr,
            timeout_receiver,
            network_task,
            network_receiver,
        );

        self.runtime = Some(runtime);

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
