// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        chained_bft_smr::{ChainedBftSMR, ChainedBftSMRConfig},
        epoch_manager::EpochManager,
        network::ConsensusNetworkImpl,
        persistent_storage::{PersistentStorage, StorageWriteProxy},
    },
    consensus_provider::ConsensusProvider,
    counters,
    state_computer::ExecutionProxy,
    state_replication::StateMachineReplication,
    txn_manager::MempoolProxy,
};
use config::config::{ConsensusProposerType::FixedProposer, NodeConfig};
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

struct InitialSetup {
    author: Author,
    signer: ValidatorSigner,
    validator: ValidatorVerifier,
}

/// Supports the implementation of ConsensusProvider using LibraBFT.
pub struct ChainedBftProvider {
    smr: ChainedBftSMR<Vec<SignedTransaction>>,
    mempool_client: Arc<MempoolClient>,
    executor: Arc<Executor<MoveVM>>,
    synchronizer_client: Arc<StateSyncClient>,
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

        let initial_setup = Self::initialize_setup(node_config);
        let epoch_mgr = Arc::new(EpochManager::new(0, initial_setup.validator.clone()));
        let network = ConsensusNetworkImpl::new(
            initial_setup.author,
            network_sender.clone(),
            network_events,
            Arc::clone(&epoch_mgr),
        );
        let proposer = {
            let peers = epoch_mgr.validators().get_ordered_account_addresses();
            if node_config.consensus.get_proposer_type() == FixedProposer {
                vec![Self::choose_leader(peers)]
            } else {
                peers
            }
        };
        debug!("[Consensus] My peer: {:?}", initial_setup.author);
        debug!("[Consensus] Chosen proposer: {:?}", proposer);
        let config = ChainedBftSMRConfig::from_node_config(&node_config.consensus);
        let (storage, initial_data) = StorageWriteProxy::start(node_config);
        info!(
            "Starting up the consensus state machine with recovery data - [consensus state {:?}], [last_vote {}], [highest timeout certificates: {}]",
            initial_data.state(),
            initial_data.last_vote().map_or("None".to_string(), |v| format!("{}", v)),
            initial_data.highest_timeout_certificates()
        );
        let smr = ChainedBftSMR::new(
            initial_setup.signer,
            proposer,
            network,
            runtime,
            config,
            storage,
            initial_data,
            epoch_mgr,
        );
        Self {
            smr,
            mempool_client,
            executor,
            synchronizer_client,
        }
    }

    /// Retrieve the initial "state" for consensus. This function is synchronous and returns after
    /// reading the local persistent store and retrieving the initial state from the executor.
    fn initialize_setup(node_config: &mut NodeConfig) -> InitialSetup {
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
            signer,
            validator,
        }
    }

    /// Choose a proposer that is going to be the single leader (relevant for a mock fixed proposer
    /// election only).
    fn choose_leader(peers: Vec<Author>) -> Author {
        // As it is just a tmp hack function, pick the max PeerId to be a proposer.
        // TODO: VRF will be integrated later.
        peers.into_iter().max().expect("No trusted peers found!")
    }
}

impl ConsensusProvider for ChainedBftProvider {
    fn start(&mut self) -> Result<()> {
        let txn_manager = Arc::new(MempoolProxy::new(self.mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(
            Arc::clone(&self.executor),
            self.synchronizer_client.clone(),
        ));
        debug!("Starting consensus provider.");
        self.smr.start(txn_manager, state_computer)
    }

    fn stop(&mut self) {
        self.smr.stop();
        debug!("Consensus provider stopped.");
    }
}
