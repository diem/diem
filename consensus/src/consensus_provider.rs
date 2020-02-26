// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        chained_bft_smr::ChainedBftSMR,
        network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
        persistent_liveness_storage::StorageWriteProxy,
    },
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
};
use anyhow::Result;
use executor::Executor;
use futures::channel::mpsc;
use libra_config::config::NodeConfig;
use libra_mempool::ConsensusRequest;
use libra_types::transaction::SignedTransaction;
use state_synchronizer::StateSyncClient;
use std::sync::Arc;
use storage_client::{StorageRead, StorageReadServiceClient};
use vm_runtime::LibraVM;

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
    network_receiver: ConsensusNetworkEvents<Vec<SignedTransaction>>,
    executor: Arc<Executor<LibraVM>>,
    state_sync_client: Arc<StateSyncClient>,
    consensus_to_mempool_sender: mpsc::Sender<ConsensusRequest>,
) -> Box<dyn ConsensusProvider> {
    let storage = Arc::new(StorageWriteProxy::new(node_config));
    let txn_manager = Box::new(MempoolProxy::new(consensus_to_mempool_sender));
    let state_computer = Arc::new(ExecutionProxy::new(executor, state_sync_client));

    Box::new(ChainedBftSMR::new(
        network_sender,
        network_receiver,
        node_config,
        state_computer,
        storage,
        txn_manager,
    ))
}

/// Create a storage read client based on the config
pub fn create_storage_read_client(config: &NodeConfig) -> Arc<dyn StorageRead> {
    Arc::new(StorageReadServiceClient::new(&config.storage.address))
}
