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
use state_synchronizer::StateSyncClient;
use std::sync::Arc;
use tokio::runtime;
use vm_runtime::LibraVM;

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
        let storage = Arc::new(StorageWriteProxy::new(node_config));
        let initial_data = runtime.block_on(storage.start());

        let mempool_client =
            MempoolClientWrapper::new("localhost", node_config.mempool.mempool_service_port);
        let txn_manager = MempoolProxy::new(mempool_client);

        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client));
        let smr = ChainedBftSMR::new(
            author,
            network_sender,
            network_events,
            node_config,
            runtime,
            storage,
            initial_data,
        );
        Self {
            smr,
            txn_manager,
            state_computer,
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
