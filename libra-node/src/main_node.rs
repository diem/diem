// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_service::start_backup_service;
use consensus::{consensus_provider::start_consensus, gen_consensus_reconfig_subscription};
use debug_interface::node_debug_service::NodeDebugService;
use executor::{db_bootstrapper::maybe_bootstrap, Executor};
use executor_types::ChunkExecutor;
use futures::{channel::mpsc::channel, executor::block_on};
use libra_config::{
    config::{NetworkConfig, NodeConfig, RoleType},
    network_id::NodeNetworkId,
    utils::get_genesis_txn,
};
use libra_json_rpc::bootstrap_from_config as bootstrap_rpc;
use libra_logger::{prelude::*, Logger};
use libra_mempool::gen_mempool_reconfig_subscription;
use libra_metrics::metric_server;
use libra_types::{
    account_config::libra_root_address, account_state::AccountState, chain_id::ChainId,
    move_resource::MoveStorage,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use network_builder::builder::NetworkBuilder;
use state_synchronizer::StateSynchronizer;
use std::{boxed::Box, convert::TryFrom, net::ToSocketAddrs, sync::Arc, thread, time::Instant};
use storage_interface::DbReaderWriter;
use storage_service::start_storage_service_with_db;
use tokio::runtime::{Builder, Runtime};

const AC_SMP_CHANNEL_BUFFER_SIZE: usize = 1_024;
const INTRA_NODE_CHANNEL_BUFFER_SIZE: usize = 1;

pub struct LibraHandle {
    _rpc: Runtime,
    _mempool: Runtime,
    _state_synchronizer: StateSynchronizer,
    network_runtimes: Vec<Runtime>,
    _consensus_runtime: Option<Runtime>,
    _debug: NodeDebugService,
    _backup: Runtime,
}

impl LibraHandle {
    pub fn shutdown(&mut self) {
        // Shutdown network runtimes to avoid panic error log after LibraHandle is dropped:
        // thread ‘network-’ panicked at ‘SelectNextSome polled after terminated’,...
        // stack backtrace:
        //    ......
        //    8: network_simple_onchain_discovery::ConfigurationChangeListener::start::{{closure}}
        //      at network/simple-onchain-discovery/src/lib.rs:175
        //    ......
        // Other runtimes don't have same problem.
        while self.network_runtimes.len() > 0 {
            self.network_runtimes.remove(0).shutdown_background();
        }
    }
}

// Fetch chain ID from on-chain resource
fn fetch_chain_id(db: &DbReaderWriter) -> ChainId {
    let blob = db
        .reader
        .get_account_state_with_proof_by_version(
            libra_root_address(),
            (&*db.reader)
                .fetch_synced_version()
                .expect("[libra-node] failed fetching synced version."),
        )
        .expect("[libra-node] failed to get Libra root address account state")
        .0
        .expect("[libra-node] missing Libra root address account state");
    AccountState::try_from(&blob)
        .expect("[libra-node] failed to convert blob to account state")
        .get_chain_id_resource()
        .expect("[libra-node] failed to get chain ID resource")
        .expect("[libra-node] missing chain ID resource")
        .chain_id()
}

fn setup_chunk_executor(db: DbReaderWriter) -> Box<dyn ChunkExecutor> {
    Box::new(Executor::<LibraVM>::new(db))
}

fn setup_debug_interface(config: &NodeConfig, logger: Option<Arc<Logger>>) -> NodeDebugService {
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

    NodeDebugService::new(addr, logger)
}

pub fn setup_environment(node_config: &NodeConfig, logger: Option<Arc<Logger>>) -> LibraHandle {
    let debug_if = setup_debug_interface(&node_config, logger);

    let metrics_port = node_config.debug_interface.metrics_server_port;
    let metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || metric_server::start_server(metric_host, metrics_port, false));
    let public_metrics_port = node_config.debug_interface.public_metrics_server_port;
    let public_metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || {
        metric_server::start_server(public_metric_host, public_metrics_port, true)
    });

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
        node_config.storage.backup_service_address,
        Arc::clone(&libra_db),
    );

    let waypoint = node_config.base.waypoint.waypoint();
    println!("WWWW1111: {}", waypoint);
    // if there's genesis txn and waypoint, commit it if the result matches.
    if let Some(genesis) = get_genesis_txn(&node_config) {
        maybe_bootstrap::<LibraVM>(&db_rw, genesis, waypoint)
            .expect("Db-bootstrapper should not fail.");
    }

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
    let chain_id = fetch_chain_id(&db_rw);
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
    if node_config.base.role.is_validator() {
        reconfig_subscriptions.push(consensus_reconfig_subscription);
    }

    // Gather all network configs into a single vector.
    // TODO:  consider explicitly encoding the role in the NetworkConfig
    let mut network_configs: Vec<(RoleType, &NetworkConfig)> = node_config
        .full_node_networks
        .iter()
        .map(|network_config| (RoleType::FullNode, network_config))
        .collect();
    if let Some(network_config) = node_config.validator_network.as_ref() {
        network_configs.push((RoleType::Validator, network_config));
    }

    let mut network_builders = Vec::new();

    // Instantiate every network and collect the requisite endpoints for state_sync, mempool, and consensus.
    for (idx, (role, network_config)) in network_configs.into_iter().enumerate() {
        // Perform common instantiation steps
        let mut network_builder = NetworkBuilder::create(chain_id, role, network_config);
        let network_id = network_config.network_id.clone();

        // Create the endpoints to connect the Network to StateSynchronizer.
        let (state_sync_sender, state_sync_events) = network_builder
            .add_protocol_handler(state_synchronizer::network::network_endpoint_config());
        state_sync_network_handles.push((
            NodeNetworkId::new(network_id.clone(), idx),
            state_sync_sender,
            state_sync_events,
        ));

        // Create the endpoints to connect the Network to mempool.
        let (mempool_sender, mempool_events) =
            network_builder.add_protocol_handler(libra_mempool::network::network_endpoint_config(
                // TODO:  Make this configuration option more clear.
                node_config.mempool.max_broadcasts_per_peer,
            ));
        mempool_network_handles.push((
            NodeNetworkId::new(network_id, idx),
            mempool_sender,
            mempool_events,
        ));

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

        reconfig_subscriptions.append(network_builder.reconfig_subscriptions());

        network_builders.push(network_builder);
    }

    // Build the configured networks.
    for network_builder in &mut network_builders {
        debug!("Creating runtime for {}", network_builder.network_context());
        let runtime = Builder::new()
            .thread_name("network-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to start runtime. Won't be able to start networking.");
        network_builder.build(runtime.handle().clone());
        network_runtimes.push(runtime);
        debug!(
            "Network built for network context: {}",
            network_builder.network_context()
        );
    }

    // Start the configured networks.
    // TODO:  Collect all component starts at the end of this function
    for network_builder in &mut network_builders {
        network_builder.start();
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

    let rpc_runtime = bootstrap_rpc(&node_config, chain_id, libra_db.clone(), mp_client_sender);

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

    LibraHandle {
        network_runtimes,
        _rpc: rpc_runtime,
        _mempool: mempool,
        _state_synchronizer: state_synchronizer,
        _consensus_runtime: consensus_runtime,
        _debug: debug_if,
        _backup: backup_service,
    }
}
