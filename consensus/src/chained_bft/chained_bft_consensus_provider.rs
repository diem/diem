// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::StateComputer;
use crate::{
    chained_bft::{
        chained_bft_smr::ChainedBftSMR,
        persistent_storage::{PersistentStorage, StorageWriteProxy},
    },
    consensus_provider::ConsensusProvider,
    state_computer::ExecutionProxy,
    state_replication::StateMachineReplication,
    txn_manager::MempoolProxy,
};
use anyhow::Result;
use executor::Executor;
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_mempool::proto::mempool_client::MempoolClientWrapper;
use libra_types::transaction::SignedTransaction;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
use safety_rules::SafetyRulesManagerConfig;
use state_synchronizer::StateSyncClient;
use std::sync::Arc;
use tokio::runtime;
use vm_runtime::LibraVM;

///  The state necessary to begin state machine replication including ValidatorSet, networking etc.
pub struct InitialSetup {
    pub network_sender: ConsensusNetworkSender,
    pub network_events: ConsensusNetworkEvents,
    pub safety_rules_manager_config: Option<SafetyRulesManagerConfig>,
}

/// Supports the implementation of ConsensusProvider using LibraBFT.
pub struct ChainedBftProvider {
    smr: ChainedBftSMR<Vec<SignedTransaction>>,
    txn_manager: MempoolProxy,
    state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
}

impl ChainedBftProvider {
    pub fn new(
        node_config: &mut NodeConfig,
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        executor: Arc<Executor<LibraVM>>,
        synchronizer_client: Arc<StateSyncClient>,
    ) -> Self {
        let mut runtime = runtime::Builder::new()
            .thread_name("consensus-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime!");

        let author = node_config.validator_network.as_ref().unwrap().peer_id;
        debug!("[Consensus] My peer: {:?}", author);
        let initial_setup = Self::initialize_setup(network_sender, network_events, node_config);
        let config = node_config.consensus.clone();
        let storage = Arc::new(StorageWriteProxy::new(node_config));
        let initial_data = runtime.block_on(storage.start());

        let mempool_client =
            MempoolClientWrapper::new("localhost", node_config.mempool.mempool_service_port);
        let txn_manager = MempoolProxy::new(mempool_client);

        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client));
        let smr = ChainedBftSMR::new(
            author,
            initial_setup,
            runtime,
            config,
            storage,
            initial_data,
        );
        Self {
            smr,
            txn_manager,
            state_computer,
        }
    }

    /// Retrieve the initial "state" for consensus. This function is synchronous and returns after
    /// reading the local persistent store and retrieving the initial state from the executor.
    fn initialize_setup(
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        node_config: &mut NodeConfig,
    ) -> InitialSetup {
        InitialSetup {
            network_sender,
            network_events,
            safety_rules_manager_config: Some(SafetyRulesManagerConfig::new(node_config)),
        }
    }
}

impl ConsensusProvider for ChainedBftProvider {
    fn start(&mut self) -> Result<()> {
        debug!("Starting consensus provider.");
        self.smr.start(
            Box::new(self.txn_manager.clone()),
            Arc::clone(&self.state_computer),
        )
    }

    fn stop(&mut self) {
        self.smr.stop();
        debug!("Consensus provider stopped.");
    }
}
