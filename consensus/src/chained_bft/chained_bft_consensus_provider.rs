// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        chained_bft_smr::ChainedBftSMR, network::ConsensusNetworkImpl,
        persistent_storage::PersistentStorage,
    },
    consensus_provider::ConsensusProvider,
    counters,
    state_computer::ExecutionProxy,
    state_replication::StateMachineReplication,
    txn_manager::MempoolProxy,
};
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
use nextgen_crypto::ed25519::*;

use crate::{
    chained_bft::{
        chained_bft_smr::ChainedBftSMRConfig, common::Author, persistent_storage::StorageWriteProxy,
    },
    state_synchronizer::{setup_state_synchronizer, StateSynchronizer},
};
use config::config::{ConsensusProposerType::FixedProposer, NodeConfig};
use execution_proto::proto::execution_grpc::ExecutionClient;
use failure::prelude::*;
use logger::prelude::*;
use mempool::proto::mempool_grpc::MempoolClient;
use std::{convert::TryFrom, sync::Arc};
use tokio::runtime;
use types::{
    account_address::AccountAddress, transaction::SignedTransaction,
    validator_signer::ValidatorSigner, validator_verifier::ValidatorVerifier,
};

struct InitialSetup {
    author: Author,
    signer: ValidatorSigner<Ed25519PrivateKey>,
    quorum_size: usize,
    peers: Arc<Vec<Author>>,
    validator: Arc<ValidatorVerifier<Ed25519PublicKey>>,
}

/// Supports the implementation of ConsensusProvider using LibraBFT.
pub struct ChainedBftProvider {
    smr: ChainedBftSMR<Vec<SignedTransaction>, Author>,
    mempool_client: Arc<MempoolClient>,
    execution_client: Arc<ExecutionClient>,
    synchronizer_client: Arc<StateSynchronizer>,
}

impl ChainedBftProvider {
    pub fn new(
        node_config: &NodeConfig,
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        mempool_client: Arc<MempoolClient>,
        execution_client: Arc<ExecutionClient>,
    ) -> Self {
        let runtime = runtime::Builder::new()
            .name_prefix("consensus-")
            .build()
            .expect("Failed to create Tokio runtime!");

        let initial_setup = Self::initialize_setup(node_config);
        let network = ConsensusNetworkImpl::new(
            initial_setup.author,
            network_sender.clone(),
            network_events,
            Arc::clone(&initial_setup.peers),
            Arc::clone(&initial_setup.validator),
        );
        let synchronizer =
            setup_state_synchronizer(network_sender, runtime.executor(), node_config);
        let proposer = {
            if node_config.consensus.get_proposer_type() == FixedProposer {
                vec![Self::choose_leader(&initial_setup)]
            } else {
                initial_setup.validator.get_ordered_account_addresses()
            }
        };
        debug!("[Consensus] My peer: {:?}", initial_setup.author);
        debug!("[Consensus] Chosen proposer: {:?}", proposer);
        let config = ChainedBftSMRConfig::from_node_config(&node_config.consensus);
        let (storage, initial_data) = StorageWriteProxy::start(node_config);
        info!(
            "Starting up the consensus state machine with recovery data - {:?}, {:?}",
            initial_data.state(),
            initial_data.highest_timeout_certificates()
        );
        let smr = ChainedBftSMR::new(
            initial_setup.author,
            initial_setup.quorum_size,
            initial_setup.signer,
            proposer,
            network,
            runtime,
            config,
            storage,
            initial_data,
        );
        Self {
            smr,
            mempool_client,
            execution_client,
            synchronizer_client: Arc::new(synchronizer),
        }
    }

    /// Retrieve the initial "state" for consensus. This function is synchronous and returns after
    /// reading the local persistent store and retrieving the initial state from the executor.
    fn initialize_setup(node_config: &NodeConfig) -> InitialSetup {
        // Keeping the initial set of validators in a node config is embarrassing and we should
        // all feel bad about it.
        let peer_id_str = node_config.base.peer_id.clone();
        let author =
            AccountAddress::try_from(peer_id_str).expect("Failed to parse peer id of a validator");
        let private_key = node_config.base.peer_keypairs.get_consensus_private();
        let _public_key = node_config.base.peer_keypairs.get_consensus_public();
        let signer = ValidatorSigner::new(author, private_key.into());
        let peers_with_public_keys = node_config.base.trusted_peers.get_trusted_consensus_peers();
        let peers_with_nextgen_public_keys = peers_with_public_keys
            .clone()
            .into_iter()
            .map(|(k, v)| (AccountAddress::clone(&k), v.into()))
            .collect();
        let peers = Arc::new(
            peers_with_public_keys
                .keys()
                .map(AccountAddress::clone)
                .collect(),
        );
        let validator = Arc::new(ValidatorVerifier::new(peers_with_nextgen_public_keys));
        counters::EPOCH_NUM.set(0); // No reconfiguration yet, so it is always zero
        counters::CURRENT_EPOCH_NUM_VALIDATORS.set(validator.len() as i64);
        counters::CURRENT_EPOCH_QUORUM_SIZE.set(validator.quorum_size() as i64);
        debug!("[Consensus]: quorum_size = {:?}", validator.quorum_size());
        InitialSetup {
            author,
            signer,
            quorum_size: validator.quorum_size(),
            peers,
            validator,
        }
    }

    /// Choose a proposer that is going to be the single leader (relevant for a mock fixed proposer
    /// election only).
    fn choose_leader(initial_setup: &InitialSetup) -> Author {
        // As it is just a tmp hack function, pick the smallest PeerId to be a proposer.
        *initial_setup
            .peers
            .iter()
            .max()
            .expect("No trusted peers found!")
    }
}

impl ConsensusProvider for ChainedBftProvider {
    fn start(&mut self) -> Result<()> {
        let txn_manager = Arc::new(MempoolProxy::new(self.mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(
            self.execution_client.clone(),
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
