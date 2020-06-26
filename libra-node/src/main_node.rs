// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_service::start_backup_service;
use consensus::{consensus_provider::start_consensus, gen_consensus_reconfig_subscription};
use debug_interface::node_debug_service::NodeDebugService;
use executor::{db_bootstrapper::bootstrap_db_if_empty, Executor};
use executor_types::ChunkExecutor;
use futures::{channel::mpsc::channel, executor::block_on};
use libra_config::{
    config::{NetworkConfig, NodeConfig, RoleType},
    utils::get_genesis_txn,
};
use libra_json_rpc::bootstrap_from_config as bootstrap_rpc;
use libra_logger::prelude::*;
use libra_mempool::gen_mempool_reconfig_subscription;
use libra_metrics::metric_server;
use libra_vm::LibraVM;
use libradb::LibraDB;
use network_builder::builder::NetworkBuilder;
use state_synchronizer::StateSynchronizer;
use std::{boxed::Box, net::ToSocketAddrs, sync::Arc, thread, time::Instant};
use storage_interface::DbReaderWriter;
use storage_service::start_storage_service_with_db;
use tokio::runtime::Runtime;

use debug_interface::libra_trace;

const AC_SMP_CHANNEL_BUFFER_SIZE: usize = 1_024;
const INTRA_NODE_CHANNEL_BUFFER_SIZE: usize = 1;

pub struct LibraHandle {
    _rpc: Runtime,
    _mempool: Runtime,
    _state_synchronizer: StateSynchronizer,
    _network_runtimes: Vec<Runtime>,
    _consensus_runtime: Option<Runtime>,
    _debug: NodeDebugService,
    _backup: Runtime,
}

fn setup_chunk_executor(db: DbReaderWriter) -> Box<dyn ChunkExecutor> {
    Box::new(Executor::<LibraVM>::new(db))
}

fn setup_debug_interface(config: &NodeConfig) -> NodeDebugService {
    let addr = format!(
        "{}:{}",
        config.debug_interface.address, config.debug_interface.admission_control_node_debug_port,
    )
    .to_socket_addrs()
    .unwrap()
    .next()
    .unwrap();

    libra_trace::set_libra_trace(&config.debug_interface.libra_trace.sampling)
        .expect("Failed to set libra trace sampling rate.");

    NodeDebugService::new(addr)
}

pub fn setup_environment(node_config: &mut NodeConfig) -> LibraHandle {
    // Some of our code uses the rayon global thread pool. Name the rayon threads so it doesn't
    // cause confusion, otherwise the threads would have their parent's name.
    rayon::ThreadPoolBuilder::new()
        .thread_name(|index| format!("rayon-global-{}", index))
        .build_global()
        .expect("Building rayon global thread pool should work.");

    let mut instant = Instant::now();
    let (libra_db, db_rw) = DbReaderWriter::wrap(
        LibraDB::open(
            &node_config.storage.dir(),
            false, /* readonly */
            node_config.storage.prune_window,
        )
        .expect("DB should open."),
    );
    let _simple_storage_service =
        start_storage_service_with_db(&node_config, Arc::clone(&libra_db));
    let backup_service = start_backup_service(
        node_config.storage.backup_service_port,
        Arc::clone(&libra_db),
    );

    bootstrap_db_if_empty::<LibraVM>(&db_rw, get_genesis_txn(&node_config).unwrap())
        .expect("Db-bootstrapper should not fail.");

    debug!(
        "Storage service started in {} ms",
        instant.elapsed().as_millis()
    );

    instant = Instant::now();
    let chunk_executor = setup_chunk_executor(db_rw.clone());
    debug!(
        "ChunkExecutor setup in {} ms",
        instant.elapsed().as_millis()
    );
    let mut network_runtimes = vec![];
    let mut state_sync_network_handles = vec![];
    let mut mempool_network_handles = vec![];
    let mut consensus_network_handles = None;
    let mut reconfig_subscriptions = vec![];

    let (mempool_reconfig_subscription, mempool_reconfig_events) =
        gen_mempool_reconfig_subscription();
    reconfig_subscriptions.push(mempool_reconfig_subscription);
    // consensus has to subscribe to ALL on-chain configs
    let (consensus_reconfig_subscription, consensus_reconfig_events) =
        gen_consensus_reconfig_subscription();
    reconfig_subscriptions.push(consensus_reconfig_subscription);

    let waypoint = node_config.base.waypoint.waypoint();

    // Gather all network configs into a single vector.
    // TODO:  consider explicitly encoding the role in the NetworkConfig
    let mut network_configs: Vec<(RoleType, &mut NetworkConfig)> = node_config
        .full_node_networks
        .iter_mut()
        .map(|network_config| (RoleType::FullNode, network_config))
        .collect();
    if let Some(network_config) = node_config.validator_network.as_mut() {
        network_configs.push((RoleType::Validator, network_config));
    }

    // Instantiate every network and collect the requisite endpoints for state_sync, mempool, and consensus.
    for (role, network_config) in network_configs {
        // Perform common instantiation steps
        let (runtime, mut network_builder) = NetworkBuilder::create(
            &node_config.base.chain_id,
            role,
            network_config,
            &mut reconfig_subscriptions,
        );
        let network_id = network_config.network_id.clone();

        // Create the endpoints to connect the Network to StateSynchronizer.
        let (state_sync_sender, state_sync_events) = network_builder
            .add_protocol_handler(state_synchronizer::network::network_endpoint_config());
        state_sync_network_handles.push((network_id.clone(), state_sync_sender, state_sync_events));

        // Create the endpoints t connect the Network to MemPool.
        let (mempool_sender, mempool_events) =
            network_builder.add_protocol_handler(libra_mempool::network::network_endpoint_config(
                // TODO:  Make this configuration option more clear.
                node_config.mempool.max_broadcasts_per_peer,
            ));
        mempool_network_handles.push((network_id, mempool_sender, mempool_events));

        match role {
            // Perform steps relevant specifically to Validator networks.
            RoleType::Validator => {
                // A valid config is allowed to have at most one ValidatorNetwork
                // TODO:  `expect_none` would be perfect here, once it is stable.
                if consensus_network_handles.is_some() {
                    panic!("There can be at most one validator network!");
                }

                consensus_network_handles =
                    Some(network_builder.add_protocol_handler(
                        consensus::network_interface::network_endpoint_config(),
                    ));
            }
            // Currently no FullNode network specific steps.
            RoleType::FullNode => (),
        }

        // Start the network and cache the runtime so it does not go out of scope.
        // TODO:  move all 'start' commands to a second phase at the end of setup_environment.  Target is to have one pass to wire the pieces together and a second pass to start processing in an appropriate order.
        let peer_id = network_builder.peer_id();
        let _listen_addr = network_builder.build();
        network_runtimes.push(runtime);
        debug!("Network started for peer_id: {}", peer_id);
    }

    // TODO set up on-chain discovery network based on UpstreamConfig.fallback_network
    // and pass network handles to mempool/state sync

    // for state sync to send requests to mempool
    let (state_sync_to_mempool_sender, state_sync_requests) =
        channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);
    let state_synchronizer = StateSynchronizer::bootstrap(
        state_sync_network_handles,
        state_sync_to_mempool_sender,
        Arc::clone(&db_rw.reader),
        chunk_executor,
        &node_config,
        waypoint,
        reconfig_subscriptions,
    );
    let (mp_client_sender, mp_client_events) = channel(AC_SMP_CHANNEL_BUFFER_SIZE);

    let rpc_runtime = bootstrap_rpc(&node_config, libra_db.clone(), mp_client_sender);

    let mut consensus_runtime = None;
    let (consensus_to_mempool_sender, consensus_requests) = channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);

    instant = Instant::now();
    let mempool = libra_mempool::bootstrap(
        node_config,
        Arc::clone(&db_rw.reader),
        mempool_network_handles,
        mp_client_events,
        consensus_requests,
        state_sync_requests,
        mempool_reconfig_events,
    );
    debug!("Mempool started in {} ms", instant.elapsed().as_millis());

    // StateSync should be instantiated and started before Consensus to avoid a cyclic dependency:
    // network provider -> consensus -> state synchronizer -> network provider.  This has resulted
    // in a deadlock as observed in GitHub issue #749.
    if let Some((consensus_network_sender, consensus_network_events)) = consensus_network_handles {
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
        consensus_runtime = Some(start_consensus(
            node_config,
            consensus_network_sender,
            consensus_network_events,
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
        _rpc: rpc_runtime,
        _mempool: mempool,
        _state_synchronizer: state_synchronizer,
        _consensus_runtime: consensus_runtime,
        _debug: debug_if,
        _backup: backup_service,
    }
}
