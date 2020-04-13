// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        chained_bft_smr::{ChainedBftSMR, ChainedBftSMRInput},
        network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
        persistent_liveness_storage::StorageWriteProxy,
    },
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
};
use anyhow::Result;
use channel::libra_channel;
use executor::Executor;
use futures::channel::mpsc;
use libra_config::config::NodeConfig;
use libra_mempool::ConsensusRequest;
use libra_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction};
use libra_vm::LibraVM;
use safety_rules::SafetyRulesManager;
use state_synchronizer::StateSyncClient;
use std::sync::{Arc, Mutex};
use storage_interface::DbReader;

/// Public interface to a consensus protocol.
pub trait ConsensusProvider {
    /// Spawns new threads, starts the consensus operations (retrieve txns, consensus protocol,
    /// execute txns, commit txns, update txn status in the mempool, etc).
    /// The function returns after consensus has recovered its initial state,
    /// and has established the required connections (e.g., to mempool and
    /// executor).
    fn start(&mut self) -> Result<()>;

    /// Stop the consensus operations. The function returns after graceful shutdown.
    fn stop(&mut self);
}

/// Helper function to create a ConsensusProvider based on configuration
pub fn make_consensus_provider(
    node_config: &mut NodeConfig,
    network_sender: ConsensusNetworkSender<Vec<SignedTransaction>>,
    network_events: ConsensusNetworkEvents<Vec<SignedTransaction>>,
    executor: Arc<Mutex<Executor<LibraVM>>>,
    state_sync_client: Arc<StateSyncClient>,
    consensus_to_mempool_sender: mpsc::Sender<ConsensusRequest>,
    libra_db: Arc<dyn DbReader>,
    reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
) -> Box<dyn ConsensusProvider> {
    let storage = Arc::new(StorageWriteProxy::new(node_config, libra_db));
    let txn_manager = Box::new(MempoolProxy::new(consensus_to_mempool_sender));
    let state_computer = Arc::new(ExecutionProxy::new(executor, state_sync_client));
    let input = ChainedBftSMRInput {
        network_sender,
        network_events,
        safety_rules_manager: SafetyRulesManager::new(node_config),
        state_computer,
        txn_manager,
        storage,
        config: node_config.consensus.clone(),
        reconfig_events,
    };

    Box::new(ChainedBftSMR::new(
        node_config.validator_network.as_ref().unwrap().peer_id,
        input,
    ))
}
