// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::NodeConfig;
use network::validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender};

use crate::chained_bft::chained_bft_consensus_provider::ChainedBftProvider;
use executor::Executor;
use grpcio::EnvBuilder;
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
) -> Box<dyn ConsensusProvider> {
    Box::new(ChainedBftProvider::new(
        node_config,
        network_sender,
        network_receiver,
        executor,
        state_sync_client,
    ))
}

/// Create a storage read client based on the config
pub fn create_storage_read_client(config: &NodeConfig) -> Arc<dyn StorageRead> {
    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-con-sto-").build());
    Arc::new(StorageReadServiceClient::new(
        env,
        &config.storage.address,
        config.storage.port,
    ))
}
