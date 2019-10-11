// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::{StateComputer, TxnManager};
use crate::{
    chained_bft::{
        chained_bft_smr::{ChainedBftSMR, ChainedBftSMRConfig},
        persistent_storage::{PersistentStorage, StorageWriteProxy},
    },
    consensus_provider::ConsensusProvider,
    counters,
    state_computer::ExecutionProxy,
    state_replication::StateMachineReplication,
    txn_manager::MempoolProxy,
};
use config::config::NodeConfig;
use consensus_types::common::Author;
use executor::Executor;
use failure::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{ValidatorSigner, ValidatorVerifier},
    transaction::SignedTransaction,
};
use logger::prelude::*;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
use state_synchronizer::StateSyncClient;
use std::{convert::TryFrom, sync::Arc};
use tokio::runtime;
use vm_runtime::MoveVM;

///  The state necessary to begin state machine replication including ValidatorSet, networking etc.
pub struct InitialSetup {
    pub author: Author,
    pub epoch: u64,
    pub signer: ValidatorSigner,
    pub validator: ValidatorVerifier,
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
            .name_prefix("consensus-")
            .build()
            .expect("Failed to create Tokio runtime!");

        let initial_setup = Self::initialize_setup(network_sender, network_events, node_config);
        debug!("[Consensus] My peer: {:?}", initial_setup.author);
        let config = ChainedBftSMRConfig::from_node_config(&node_config.consensus);
        let (storage, initial_data) = StorageWriteProxy::start(node_config);
        info!(
            "Starting up the consensus state machine with recovery data - [consensus state {:?}], [last_vote {}], [highest timeout certificate: {}]",
            initial_data.state(),
            initial_data.last_vote().map_or("None".to_string(), |v| v.to_string()),
            initial_data.highest_timeout_certificate().map_or("None".to_string(), |v| v.to_string()),
        );
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
        let peer_id_str = node_config
            .get_validator_network_config()
            .unwrap()
            .peer_id
            .clone();
        let author =
            AccountAddress::try_from(peer_id_str).expect("Failed to parse peer id of a validator");
        let private_key = node_config
            .consensus
            .consensus_keypair
            .take_consensus_private()
            .expect(
            "Failed to move a Consensus private key from a NodeConfig, key absent or already read",
        );
        let signer = ValidatorSigner::new(author, private_key);
        // Keeping the initial set of validators in a node config is embarrassing and we should
        // all feel bad about it.
        let validator = node_config
            .consensus
            .consensus_peers
            .get_validator_verifier();
        counters::EPOCH_NUM.set(0); // No reconfiguration yet, so it is always zero
        counters::CURRENT_EPOCH_NUM_VALIDATORS.set(validator.len() as i64);
        counters::CURRENT_EPOCH_QUORUM_SIZE.set(validator.quorum_voting_power() as i64);
        debug!(
            "[Consensus]: quorum_size = {:?}",
            validator.quorum_voting_power()
        );
        InitialSetup {
            author,
            // TODO: this is placeholder for now, replace with reconfiguration
            epoch: 0,
            signer,
            validator,
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
