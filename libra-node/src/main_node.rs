// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_service::admission_control_service::AdmissionControlService;
use consensus::consensus_provider::{make_consensus_provider, ConsensusProvider};
use debug_interface::{
    node_debug_service::NodeDebugService,
    proto::node_debug_interface_server::NodeDebugInterfaceServer,
};
use executor::{db_bootstrapper::maybe_bootstrap_db, Executor};
use futures::{channel::mpsc::channel, executor::block_on};
use libra_config::config::{NetworkConfig, NodeConfig, RoleType};
use libra_json_rpc::bootstrap_from_config as bootstrap_rpc;
use libra_logger::prelude::*;
use libra_mempool::MEMPOOL_SUBSCRIBED_CONFIGS;
use libra_metrics::metric_server;
use libra_types::event_subscription::ReconfigSubscription;
use libra_vm::LibraVM;
use network::validator_network::network_builder::{NetworkBuilder, TransportType};
use state_synchronizer::StateSynchronizer;
use std::{
    collections::HashMap,
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};
use storage_client::SyncStorageClient;
use storage_service::{init_libra_db, start_storage_service_with_db};
use tokio::runtime::{Builder, Runtime};

const AC_SMP_CHANNEL_BUFFER_SIZE: usize = 1_024;
const INTRA_NODE_CHANNEL_BUFFER_SIZE: usize = 1;

pub struct LibraHandle {
    _ac: Runtime,
    _rpc: Runtime,
    _mempool: Runtime,
    _state_synchronizer: StateSynchronizer,
    _network_runtimes: Vec<Runtime>,
    consensus: Option<Box<dyn ConsensusProvider>>,
    _storage: Runtime,
    _debug: Runtime,
}

impl Drop for LibraHandle {
    fn drop(&mut self) {
        if let Some(consensus) = &mut self.consensus {
            consensus.stop();
        }
    }
}

fn setup_executor(config: &NodeConfig) -> Arc<Mutex<Executor<LibraVM>>> {
    Arc::new(Mutex::new(Executor::new(
        SyncStorageClient::new(&config.storage.address).into(),
    )))
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

// TODO(abhayb): Move to network crate (similar to consensus).
pub fn setup_network(config: &mut NetworkConfig, role: RoleType) -> (Runtime, NetworkBuilder) {
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
        let signing_keys = &mut network_keypairs.signing_keys;
        let identity_keys = &mut network_keypairs.identity_keys;

        let signing_private = signing_keys
            .take_private()
            .expect("Failed to take Network signing private key, key absent or already read");
        let signing_public = signing_keys.public().clone();
        let identity_private = identity_keys
            .take_private()
            .expect("Failed to take Network identity private key, key absent or already read");
        let identity_public = identity_keys.public().clone();
        let trusted_peers = if role == RoleType::Validator {
            // for validators, trusted_peers is empty will be populated from consensus
            HashMap::new()
        } else {
            config.network_peers.peers.clone()
        };
        network_builder
            .transport(TransportType::TcpNoise(Some((
                identity_private,
                identity_public,
            ))))
            .connectivity_check_interval_ms(config.connectivity_check_interval_ms)
            .seed_peers(seed_peers)
            .trusted_peers(trusted_peers)
            .signing_keys((signing_private, signing_public))
            .discovery_interval_ms(config.discovery_interval_ms)
            .add_discovery();
    } else if config.enable_noise {
        let identity_keys = &mut config
            .network_keypairs
            .as_mut()
            .expect("Network keypairs are not defined")
            .identity_keys;
        let identity_private = identity_keys
            .take_private()
            .expect("Failed to take Network identity private key, key absent or already read");
        let identity_public = identity_keys.public().clone();
        // Even if a network end-point operates without remote authentication, it might want to prove
        // its identity to another peer it connects to. For this, we use TCP + Noise but without
        // enforcing a trusted peers set.
        network_builder.transport(TransportType::TcpNoise(Some((
            identity_private,
            identity_public,
        ))));
    } else {
        network_builder.transport(TransportType::Tcp);
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
    let (libra_db, db_reader_writer) = init_libra_db(&node_config);
    let storage = start_storage_service_with_db(&node_config, Arc::clone(&libra_db));
    maybe_bootstrap_db::<LibraVM>(db_reader_writer, node_config)
        .expect("Db-bootstrapper should not fail.");

    debug!(
        "Storage service started in {} ms",
        instant.elapsed().as_millis()
    );

    instant = Instant::now();
    let executor = setup_executor(&node_config);
    debug!("Executor setup in {} ms", instant.elapsed().as_millis());
    let mut network_runtimes = vec![];
    let mut state_sync_network_handles = vec![];
    let mut mempool_network_handles = vec![];
    let mut validator_network_provider = None;
    let mut reconfig_subscriptions = vec![];

    let (mempool_reconfig_subscription, mempool_reconfig_events) =
        ReconfigSubscription::subscribe(MEMPOOL_SUBSCRIBED_CONFIGS);
    reconfig_subscriptions.push(mempool_reconfig_subscription);

    if let Some(network) = node_config.validator_network.as_mut() {
        let (runtime, mut network_builder) = setup_network(network, RoleType::Validator);
        state_sync_network_handles.push(state_synchronizer::network::add_to_network(
            &mut network_builder,
        ));

        let (mempool_sender, mempool_events) =
            libra_mempool::network::add_to_network(&mut network_builder);
        mempool_network_handles.push((network.peer_id, mempool_sender, mempool_events));
        validator_network_provider = Some((network.peer_id, runtime, network_builder));
    }

    for mut full_node_network in node_config.full_node_networks.iter_mut() {
        let (runtime, mut network_builder) =
            setup_network(&mut full_node_network, RoleType::FullNode);

        network_runtimes.push(runtime);
        state_sync_network_handles.push(state_synchronizer::network::add_to_network(
            &mut network_builder,
        ));
        let (mempool_sender, mempool_events) =
            libra_mempool::network::add_to_network(&mut network_builder);
        mempool_network_handles.push((full_node_network.peer_id, mempool_sender, mempool_events));

        // Start the network provider.
        let _listen_addr = network_builder.build();
        debug!("Network started for peer_id: {}", full_node_network.peer_id);
    }

    // for state sync to send requests to mempool
    let (state_sync_to_mempool_sender, state_sync_requests) =
        channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);
    let state_synchronizer = StateSynchronizer::bootstrap(
        state_sync_network_handles,
        state_sync_to_mempool_sender,
        Arc::clone(&executor),
        &node_config,
        reconfig_subscriptions,
    );
    let (mp_client_sender, mp_client_events) = channel(AC_SMP_CHANNEL_BUFFER_SIZE);

    let admission_control_runtime =
        AdmissionControlService::bootstrap(&node_config, mp_client_sender.clone());
    let rpc_runtime = bootstrap_rpc(&node_config, libra_db.clone(), mp_client_sender);

    let mut consensus = None;
    let (consensus_to_mempool_sender, consensus_requests) = channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);

    instant = Instant::now();
    let mempool = libra_mempool::bootstrap(
        node_config,
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

        // Initialize and start consensus.
        instant = Instant::now();
        let mut consensus_provider = make_consensus_provider(
            node_config,
            consensus_network_sender,
            consensus_network_events,
            executor,
            state_synchronizer.create_client(),
            consensus_to_mempool_sender,
            libra_db,
        );
        consensus_provider
            .start()
            .expect("Failed to start consensus. Can't proceed.");
        consensus = Some(consensus_provider);
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
        consensus,
        _storage: storage,
        _debug: debug_if,
    }
}
