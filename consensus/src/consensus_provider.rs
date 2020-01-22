// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{chained_bft_smr::ChainedBftSMR, persistent_storage::StorageWriteProxy},
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
};
use anyhow::Result;
use executor::Executor;
use libra_config::config::NodeConfig;
use libra_mempool::proto::mempool_client::MempoolClientWrapper;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};
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
    network_sender: ConsensusNetworkSender,
    network_receiver: ConsensusNetworkEvents,
    executor: Arc<Executor<LibraVM>>,
    state_sync_client: Arc<StateSyncClient>,
    rt: &mut tokio::runtime::Runtime,
) -> Box<dyn ConsensusProvider> {
    let storage = Arc::new(StorageWriteProxy::new(node_config, rt));
    let mempool_client =
        MempoolClientWrapper::new("localhost", node_config.mempool.mempool_service_port);
    let txn_manager = Box::new(MempoolProxy::new(mempool_client));
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
pub fn create_storage_read_client(
    config: &NodeConfig,
    rt: &mut tokio::runtime::Runtime,
) -> Arc<dyn StorageRead> {
    Arc::new(StorageReadServiceClient::new(
        &config.storage.address,
        config.storage.port,
        rt,
    ))
}
