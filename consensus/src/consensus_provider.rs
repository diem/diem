// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::NodeConfig;
use network::validator_network::{
    ChainStateNetworkEvents, ChainStateNetworkSender, ConsensusNetworkEvents,
    ConsensusNetworkSender,
};

use crate::chained_bft::chained_bft_consensus_provider::ChainedBftProvider;
use crate::pow::pow_consensus_provider::PowConsensusProvider;
use executor::Executor;
use grpcio::{ChannelBuilder, EnvBuilder};
use libra_mempool::proto::mempool::MempoolClient;
use state_synchronizer::StateSyncClient;
use std::sync::Arc;
use storage_client::{
    StorageRead, StorageReadServiceClient, StorageWrite, StorageWriteServiceClient,
};
use vm_runtime::MoveVM;

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
    executor: Arc<Executor<MoveVM>>,
    state_sync_client: Arc<StateSyncClient>,
) -> Box<dyn ConsensusProvider> {
    Box::new(ChainedBftProvider::new(
        node_config,
        network_sender,
        network_receiver,
        create_mempool_client(node_config),
        executor,
        state_sync_client,
    ))
}

/// Create a mempool client assuming the mempool is running on localhost
fn create_mempool_client(config: &NodeConfig) -> Arc<MempoolClient> {
    let port = config.mempool.mempool_service_port;
    let connection_str = format!("localhost:{}", port);

    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-con-mem-").build());
    Arc::new(MempoolClient::new(
        ChannelBuilder::new(env).connect(&connection_str),
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

/// Create a storage write client based on the config
pub fn create_storage_write_client(config: &NodeConfig) -> Arc<dyn StorageWrite> {
    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-con-sto-").build());
    Arc::new(StorageWriteServiceClient::new(
        Arc::clone(&env),
        &config.storage.address,
        config.storage.port,
        config.storage.grpc_max_receive_len,
    ))
}

/// pow provider
pub fn make_pow_consensus_provider(
    node_config: &mut NodeConfig,
    network_sender: ConsensusNetworkSender,
    network_receiver: ConsensusNetworkEvents,
    executor: Arc<Executor<MoveVM>>,
    state_sync_client: Arc<StateSyncClient>,
    rollback_flag: bool,
    chain_state_network_sender: ChainStateNetworkSender,
    chain_state_network_events: ChainStateNetworkEvents,
) -> Box<dyn ConsensusProvider> {
    let read = create_storage_read_client(node_config);
    let write = create_storage_write_client(node_config);
    Box::new(PowConsensusProvider::new(
        node_config,
        network_sender,
        network_receiver,
        create_mempool_client(node_config),
        executor,
        state_sync_client,
        rollback_flag,
        read,
        write,
        chain_state_network_sender,
        chain_state_network_events,
    ))
}
