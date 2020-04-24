// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_service::admission_control_service::AdmissionControlService;
use consensus::consensus_provider::start_consensus;
use debug_interface::{
    node_debug_service::NodeDebugService,
    proto::node_debug_interface_server::NodeDebugInterfaceServer,
};
use executor::{db_bootstrapper::bootstrap_db_if_empty, BlockExecutor, ChunkExecutor, Executor};
use futures::{channel::mpsc::channel, executor::block_on, stream::StreamExt};
use libra_config::{
    config::{DiscoveryMethod, NetworkConfig, NodeConfig, RoleType},
    utils::get_genesis_txn,
};
use libra_json_rpc::bootstrap_from_config as bootstrap_rpc;
use libra_logger::prelude::*;
use libra_mempool::MEMPOOL_SUBSCRIBED_CONFIGS;
use libra_metrics::metric_server;
use libra_network_address::NetworkAddress;
use libra_types::{on_chain_config::ON_CHAIN_CONFIG_REGISTRY, waypoint::Waypoint, PeerId};
use libra_vm::LibraVM;
use network::validator_network::network_builder::{NetworkBuilder, TransportType};
use onchain_discovery::OnchainDiscovery;
use simple_storage_client::SimpleStorageClient;
use state_synchronizer::StateSynchronizer;
use std::{
    boxed::Box,
    collections::HashMap,
    net::ToSocketAddrs,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use storage_client::StorageReadServiceClient;
use storage_interface::{DbReader, DbReaderWriter};
use storage_service::{
    init_libra_db, start_simple_storage_service_with_db, start_storage_service_with_db,
};
use subscription_service::ReconfigSubscription;
use tokio::{
    runtime::{Builder, Handle, Runtime},
    time::interval,
};

const AC_SMP_CHANNEL_BUFFER_SIZE: usize = 1_024;
const INTRA_NODE_CHANNEL_BUFFER_SIZE: usize = 1;

pub struct LibraHandle {
    _ac: Runtime,
    _rpc: Runtime,
    _mempool: Runtime,
    _state_synchronizer: StateSynchronizer,
    _network_runtimes: Vec<Runtime>,
    _consensus_runtime: Option<Runtime>,
    _storage: Runtime,
    _debug: Runtime,
}

fn setup_chunk_executor(db: DbReaderWriter) -> Box<dyn ChunkExecutor> {
    Box::new(Executor::<LibraVM>::new(db))
}

fn setup_block_executor(config: &NodeConfig) -> Box<dyn BlockExecutor> {
    Box::new(Executor::<LibraVM>::new(
        SimpleStorageClient::new(&config.storage.simple_address).into(),
    ))
}

fn setup_debug_interface(config: &NodeConfig) -> Runtime {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let addr = format!(
        "{}:{}",
        config.debug_interface.address, config.debug_interface.admission_control_node_debug_port,
    )
    .to_socket_addrs()
    .unwrap()
    .next()
    .unwrap();
    rt.spawn(
        tonic::transport::Server::builder()
            .add_service(NodeDebugInterfaceServer::new(NodeDebugService::new()))
            .serve(addr),
    );
    rt
}

pub fn setup_onchain_discovery(
    network: &mut NetworkBuilder,
    peer_id: PeerId,
    role: RoleType,
    storage_read_client: Arc<StorageReadServiceClient>,
    waypoint: Waypoint,
    executor: Handle,
) {
    let executor_clone = executor.clone();
    let (network_tx, network_rx) = onchain_discovery::network_interface::add_to_network(network);
    let outbound_rpc_timeout = Duration::from_secs(30);
    let max_concurrent_inbound_queries = 8;

    let onchain_discovery = executor.enter(move || {
        let peer_query_ticker = interval(Duration::from_secs(30)).fuse();
        let storage_query_ticker = interval(Duration::from_secs(30)).fuse();

        OnchainDiscovery::new(
            executor_clone,
            peer_id,
            role,
            waypoint,
            network_tx,
            network_rx,
            storage_read_client,
            peer_query_ticker,
            storage_query_ticker,
            outbound_rpc_timeout,
            max_concurrent_inbound_queries,
        )
    });

    executor.spawn(onchain_discovery.start());
}

// TODO(abhayb): Move to network crate (similar to consensus).
pub fn setup_network(
    config: &mut NetworkConfig,
    role: RoleType,
    storage_read: Arc<StorageReadServiceClient>,
    waypoint: Waypoint,
) -> (Runtime, NetworkBuilder) {
    let runtime = Builder::new()
        .thread_name("network-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("Failed to start runtime. Won't be able to start networking.");
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        config.peer_id,
        config.listen_address.clone(),
        role,
    );
    network_builder
        .enable_remote_authentication(config.enable_remote_authentication)
        .advertised_address(config.advertised_address.clone())
        .add_connection_monitoring();
    if config.enable_remote_authentication {
        // If the node wants to run in permissioned mode, it should also have authentication and
        // encryption.
        assert!(
            config.enable_noise,
            "Permissioned network end-points should use authentication"
        );
        let seed_peers = config.seed_peers.seed_peers.clone();
        let network_keypairs = config
            .network_keypairs
            .as_mut()
            .expect("Network keypairs are not defined");
        let signing_keypair = &mut network_keypairs.signing_keypair;
        let signing_private = signing_keypair
            .take_private()
            .expect("Failed to take Network signing private key, key absent or already read");
        let signing_public = signing_keypair.public_key();

        let identity_key = network_keypairs
            .identity_keypair
            .take_private()
            .expect("identity key should be present");

        let trusted_peers = if role == RoleType::Validator {
            // for validators, trusted_peers is empty will be populated from consensus
            HashMap::new()
        } else {
            config.network_peers.peers.clone()
        };
        network_builder
            .transport(TransportType::TcpNoise(Some(identity_key)))
            .connectivity_check_interval_ms(config.connectivity_check_interval_ms)
            .seed_peers(seed_peers)
            .trusted_peers(trusted_peers)
            .signing_keypair((signing_private, signing_public))
            .discovery_interval_ms(config.discovery_interval_ms)
            .add_connectivity_manager();
    } else if config.enable_noise {
        let identity_key = config
            .network_keypairs
            .as_mut()
            .expect("Network keypairs are not defined")
            .identity_keypair
            .take_private()
            .expect("identity key should be present");
        // Even if a network end-point operates without remote authentication, it might want to prove
        // its identity to another peer it connects to. For this, we use TCP + Noise but without
        // enforcing a trusted peers set.
        network_builder.transport(TransportType::TcpNoise(Some(identity_key)));
    } else {
        network_builder.transport(TransportType::Tcp);
    }

    match config.discovery_method {
        DiscoveryMethod::Gossip => {
            network_builder.add_gossip_discovery();
        }
        DiscoveryMethod::Onchain => {
            setup_onchain_discovery(
                &mut network_builder,
                config.peer_id,
                role,
                storage_read,
                waypoint,
                runtime.handle().clone(),
            );
        }
        DiscoveryMethod::None => {}
    }

    (runtime, network_builder)
}

pub fn setup_environment(node_config: &mut NodeConfig) -> LibraHandle {
    crash_handler::setup_panic_handler();

    // Some of our code uses the rayon global thread pool. Name the rayon threads so it doesn't
    // cause confusion, otherwise the threads would have their parent's name.
    rayon::ThreadPoolBuilder::new()
        .thread_name(|index| format!("rayon-global-{}", index))
        .build_global()
        .expect("Building rayon global thread pool should work.");

    let mut instant = Instant::now();
    let (libra_db, db_rw) = init_libra_db(&node_config);
    let _simple_storage_service =
        start_simple_storage_service_with_db(&node_config, Arc::clone(&libra_db));

    // Will be deprecated. Do not reference it anymore.
    let storage = start_storage_service_with_db(&node_config, Arc::clone(&libra_db));
    bootstrap_db_if_empty::<LibraVM>(&db_rw, get_genesis_txn(&node_config).unwrap())
        .expect("Db-bootstrapper should not fail.");

    debug!(
        "Storage service started in {} ms",
        instant.elapsed().as_millis()
    );

    instant = Instant::now();
    let chunk_executor = setup_chunk_executor(db_rw);
    debug!(
        "ChunkExecutor setup in {} ms",
        instant.elapsed().as_millis()
    );
    let mut network_runtimes = vec![];
    let mut state_sync_network_handles = vec![];
    let mut mempool_network_handles = vec![];
    let mut validator_network_provider = None;
    let mut reconfig_subscriptions = vec![];

    let (mempool_reconfig_subscription, mempool_reconfig_events) =
        ReconfigSubscription::subscribe(MEMPOOL_SUBSCRIBED_CONFIGS);
    reconfig_subscriptions.push(mempool_reconfig_subscription);
    // consensus has to subscribe to ALL on-chain configs
    let (consensus_reconfig_subscription, consensus_reconfig_events) =
        ReconfigSubscription::subscribe(ON_CHAIN_CONFIG_REGISTRY);
    reconfig_subscriptions.push(consensus_reconfig_subscription);

    let storage_read = Arc::new(StorageReadServiceClient::new(&node_config.storage.address));

    if let Some(network) = node_config.validator_network.as_mut() {
        let (runtime, mut network_builder) = setup_network(
            network,
            RoleType::Validator,
            Arc::clone(&storage_read),
            node_config.base.waypoint.expect("No waypoint in config"),
        );

        let (state_sync_sender, state_sync_events) =
            state_synchronizer::network::add_to_network(&mut network_builder);
        state_sync_network_handles.push((network.peer_id, state_sync_sender, state_sync_events));

        let (mempool_sender, mempool_events) =
            libra_mempool::network::add_to_network(&mut network_builder);
        mempool_network_handles.push((network.peer_id, mempool_sender, mempool_events));
        validator_network_provider = Some((network.peer_id, runtime, network_builder));
    }

    for mut full_node_network in node_config.full_node_networks.iter_mut() {
        let (runtime, mut network_builder) = setup_network(
            &mut full_node_network,
            RoleType::FullNode,
            Arc::clone(&storage_read),
            node_config.base.waypoint.expect("No waypoint in config"),
        );

        network_runtimes.push(runtime);
        let (state_sync_sender, state_sync_events) =
            state_synchronizer::network::add_to_network(&mut network_builder);
        state_sync_network_handles.push((
            full_node_network.peer_id,
            state_sync_sender,
            state_sync_events,
        ));
        let (mempool_sender, mempool_events) =
            libra_mempool::network::add_to_network(&mut network_builder);
        mempool_network_handles.push((full_node_network.peer_id, mempool_sender, mempool_events));

        // Start the network provider.
        let _listen_addr = network_builder.build();
        debug!("Network started for peer_id: {}", full_node_network.peer_id);
    }

    // TODO set up on-chain discovery network based on UpstreamConfig.fallback_network
    // and pass network handles to mempool/state sync

    // for state sync to send requests to mempool
    let (state_sync_to_mempool_sender, state_sync_requests) =
        channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);
    let state_synchronizer = StateSynchronizer::bootstrap(
        state_sync_network_handles,
        state_sync_to_mempool_sender,
        Arc::clone(&libra_db) as Arc<dyn DbReader>,
        chunk_executor,
        &node_config,
        reconfig_subscriptions,
    );
    let (mp_client_sender, mp_client_events) = channel(AC_SMP_CHANNEL_BUFFER_SIZE);

    let admission_control_runtime = AdmissionControlService::bootstrap(
        &node_config,
        Arc::clone(&libra_db) as Arc<dyn DbReader>,
        mp_client_sender.clone(),
    );
    let rpc_runtime = bootstrap_rpc(&node_config, libra_db.clone(), mp_client_sender);

    let mut consensus_runtime = None;
    let (consensus_to_mempool_sender, consensus_requests) = channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);

    instant = Instant::now();
    let mempool = libra_mempool::bootstrap(
        node_config,
        Arc::clone(&libra_db) as Arc<dyn DbReader>,
        mempool_network_handles,
        mp_client_events,
        consensus_requests,
        state_sync_requests,
        mempool_reconfig_events,
    );
    debug!("Mempool started in {} ms", instant.elapsed().as_millis());

    if let Some((peer_id, runtime, mut network_builder)) = validator_network_provider {
        // Note: We need to start network provider before consensus, because the consensus
        // initialization is blocked on state synchronizer to sync to the initial root ledger
        // info, which in turn cannot make progress before network initialization
        // because the NewPeer events which state synchronizer uses to know its
        // peers are delivered by network provider. If we were to start network
        // provider after consensus, we create a cyclic dependency from
        // network provider -> consensus -> state synchronizer -> network provider. This deadlock
        // was observed in GitHub Issue #749. A long term fix might be make
        // consensus initialization async instead of blocking on state synchronizer.
        let (consensus_network_sender, consensus_network_events) =
            consensus::network_interface::add_to_network(&mut network_builder);
        let _listen_addr = network_builder.build();
        network_runtimes.push(runtime);
        debug!("Network started for peer_id: {}", peer_id);

        // Make sure that state synchronizer is caught up at least to its waypoint
        // (in case it's present). There is no sense to start consensus prior to that.
        // TODO: Note that we need the networking layer to be able to discover & connect to the
        // peers with potentially outdated network identity public keys.
        debug!("Wait until state synchronizer is initialized");
        block_on(state_synchronizer.wait_until_initialized())
            .expect("State synchronizer initialization failure");
        debug!("State synchronizer initialization complete.");

        instant = Instant::now();
        let block_executor = setup_block_executor(&node_config);
        debug!(
            "BlockExecutor setup in {} ms",
            instant.elapsed().as_millis()
        );

        // Initialize and start consensus.
        instant = Instant::now();
        consensus_runtime = Some(start_consensus(
            node_config,
            consensus_network_sender,
            consensus_network_events,
            block_executor,
            state_synchronizer.create_client(),
            consensus_to_mempool_sender,
            libra_db,
            consensus_reconfig_events,
        ));
        debug!("Consensus started in {} ms", instant.elapsed().as_millis());
    }

    let debug_if = setup_debug_interface(&node_config);

    let metrics_port = node_config.debug_interface.metrics_server_port;
    let metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || metric_server::start_server(metric_host, metrics_port, false));
    let public_metrics_port = node_config.debug_interface.public_metrics_server_port;
    let public_metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || {
        metric_server::start_server(public_metric_host, public_metrics_port, true)
    });

    LibraHandle {
        _network_runtimes: network_runtimes,
        _ac: admission_control_runtime,
        _rpc: rpc_runtime,
        _mempool: mempool,
        _state_synchronizer: state_synchronizer,
        _consensus_runtime: consensus_runtime,
        _storage: storage,
        _debug: debug_if,
    }
}
