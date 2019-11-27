// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::StateComputer;
use crate::txn_manager::TxnManager;
use crate::{
    chained_bft::{
        chained_bft_smr::{ChainedBftSMR, ChainedBftSMRConfig},
        persistent_storage::{PersistentStorage, StorageWriteProxy},
    },
    consensus_provider::ConsensusProvider,
    state_computer::ExecutionProxy,
    state_replication::StateMachineReplication,
    txn_manager::MempoolProxy,
};
use consensus_types::common::Author;
use executor::Executor;
use failure::prelude::*;
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use libra_types::{crypto_proxies::ValidatorSigner, transaction::SignedTransaction};
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
use state_synchronizer::StateSyncClient;
use std::sync::Arc;
use tokio::runtime;
use vm_runtime::MoveVM;

///  The state necessary to begin state machine replication including ValidatorSet, networking etc.
pub struct InitialSetup {
    pub author: Author,
    pub signer: ValidatorSigner,
    pub network_sender: ConsensusNetworkSender,
    pub network_events: ConsensusNetworkEvents,
}

/// Supports the implementation of ConsensusProvider using LibraBFT.
pub struct ChainedBftProvider {
    smr: ChainedBftSMR<Vec<SignedTransaction>>,
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
}

impl ChainedBftProvider {
    pub fn new(
        node_config: &mut NodeConfig,
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        mempool_client: Arc<MempoolClient>,
        executor: Arc<Executor<MoveVM>>,
        synchronizer_client: Arc<StateSyncClient>,
    ) -> Self {
        let runtime = runtime::Builder::new()
            .thread_name("consensus-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime!");

        let initial_setup = Self::initialize_setup(network_sender, network_events, node_config);
        debug!("[Consensus] My peer: {:?}", initial_setup.author);
        let config = ChainedBftSMRConfig::from_node_config(&node_config.consensus);
        let storage = Arc::new(StorageWriteProxy::new(node_config));
        let initial_data = storage.start();
        let txn_manager = Arc::new(MempoolProxy::new(mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client.clone()));
        let smr = ChainedBftSMR::new(initial_setup, runtime, config, storage, initial_data);
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
        let author = node_config.validator_network.as_ref().unwrap().peer_id;
        let private_key = node_config
            .consensus
            .consensus_keypair
            .take_consensus_private()
            .expect(
            "Failed to move a Consensus private key from a NodeConfig, key absent or already read",
        );
        let signer = ValidatorSigner::new(author, private_key);
        InitialSetup {
            author,
            signer,
            network_sender,
            network_events,
        }
    }
}

impl ConsensusProvider for ChainedBftProvider {
    fn start(&mut self) -> Result<()> {
        debug!("Starting consensus provider.");
        self.smr.start(
            Arc::clone(&self.txn_manager),
            Arc::clone(&self.state_computer),
        )
    }

    fn stop(&mut self) {
        self.smr.stop();
        debug!("Consensus provider stopped.");
    }
}
