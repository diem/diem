// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_service::start_backup_service;
use consensus::consensus_provider::start_consensus;
use debug_interface::node_debug_service::NodeDebugService;
use diem_api::runtime::bootstrap as bootstrap_api;
use diem_config::{
    config::{NetworkConfig, NodeConfig, PersistableConfig, StorageServiceConfig},
    network_id::NetworkId,
    utils::get_genesis_txn,
};
use diem_data_client::diemnet::DiemNetDataClient;
use diem_infallible::RwLock;
use diem_json_rpc::bootstrap_from_config as bootstrap_rpc;
use diem_logger::{prelude::*, Logger};
use diem_metrics::metric_server;
use diem_time_service::TimeService;
use diem_types::{
    account_config::diem_root_address,
    account_state::AccountState,
    chain_id::ChainId,
    move_resource::MoveStorage,
    on_chain_config::{VMPublishingOption, ON_CHAIN_CONFIG_REGISTRY},
    protocol_spec::DpnProto,
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use event_notifications::EventSubscriptionService;
use executor::{db_bootstrapper::maybe_bootstrap, Executor};
use executor_types::ChunkExecutor;
use futures::channel::mpsc::channel;
use network::application::storage::PeerMetadataStorage;
use network_builder::builder::NetworkBuilder;
use state_sync_multiplexer::{state_sync_v1_network_config, StateSyncMultiplexer};
use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    convert::TryFrom,
    io::Write,
    net::ToSocketAddrs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};
use storage_interface::default_protocol::DbReaderWriter;
use storage_service::start_storage_service_with_db;
use storage_service_client::{StorageServiceClient, StorageServiceMultiSender};
use storage_service_server::{
    network::StorageServiceNetworkEvents, StorageReader, StorageServiceServer,
};
use tokio::runtime::{Builder, Handle, Runtime};
use tokio_stream::wrappers::IntervalStream;

const AC_SMP_CHANNEL_BUFFER_SIZE: usize = 1_024;
const INTRA_NODE_CHANNEL_BUFFER_SIZE: usize = 1;
const MEMPOOL_NETWORK_CHANNEL_BUFFER_SIZE: usize = 1_024;

pub struct DiemHandle {
    _api: Runtime,
    _backup: Runtime,
    _consensus_runtime: Option<Runtime>,
    _debug: NodeDebugService,
    _mempool: Runtime,
    _network_runtimes: Vec<Runtime>,
    _state_sync: StateSyncMultiplexer,
    _storage_service: Runtime,
}

pub fn start(config: &NodeConfig, log_file: Option<PathBuf>) {
    crash_handler::setup_panic_handler();

    let mut logger = diem_logger::Logger::new();
    logger
        .channel_size(config.logger.chan_size)
        .is_async(config.logger.is_async)
        .level(config.logger.level)
        .read_env();
    if let Some(log_file) = log_file {
        logger.printer(Box::new(FileWriter::new(log_file)));
    }
    let logger = Some(logger.build());

    // Let's now log some important information, since the logger is set up
    info!(config = config, "Loaded DiemNode config");

    if fail::has_failpoints() {
        warn!("Failpoints is enabled");
        if let Some(failpoints) = &config.failpoints {
            for (point, actions) in failpoints {
                fail::cfg(point, actions).expect("fail to set actions for failpoint");
            }
        }
    } else if config.failpoints.is_some() {
        warn!("failpoints is set in config, but the binary doesn't compile with this feature");
    }

    let _node_handle = setup_environment(config, logger);
    let term = Arc::new(AtomicBool::new(false));

    while !term.load(Ordering::Acquire) {
        std::thread::park();
    }
}

pub fn load_test_environment<R>(
    config_path: Option<PathBuf>,
    random_ports: bool,
    lazy: bool,
    publishing_option: Option<VMPublishingOption>,
    genesis_modules: Vec<Vec<u8>>,
    rng: R,
) where
    R: ::rand::RngCore + ::rand::CryptoRng,
{
    // Either allocate a temppath or reuse the passed in path and make sure the directory exists
    let config_temp_path = diem_temppath::TempPath::new();
    let config_path = config_path.unwrap_or_else(|| config_temp_path.as_ref().to_path_buf());
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&config_path)
        .unwrap();
    let config_path = config_path.canonicalize().unwrap();

    // Build a single validator network
    let mut maybe_config = PathBuf::from(&config_path);
    maybe_config.push("validator_node_template.yaml");
    let mut template = NodeConfig::load_config(maybe_config)
        .unwrap_or_else(|_| NodeConfig::default_for_validator());

    // enable REST and JSON-RPC API
    template.json_rpc.address = format!("0.0.0.0:{}", template.json_rpc.address.port())
        .parse()
        .unwrap();
    template.api.address = template.json_rpc.address;
    template.json_rpc.stream_rpc.enabled = true;
    if lazy {
        template.consensus.mempool_poll_count = u64::MAX;
    }

    let mut builder =
        diem_genesis_tool::validator_builder::ValidatorBuilder::new(&config_path, genesis_modules)
            .template(template)
            .randomize_first_validator_ports(random_ports);
    if let Some(publishing_option) = publishing_option {
        builder = builder.publishing_option(publishing_option);
    }
    let (root_keys, _genesis, genesis_waypoint, validators) = builder.build(rng).unwrap();

    let diem_root_key_path = config_path.join("mint.key");
    let serialized_keys = bcs::to_bytes(&root_keys.root_key).unwrap();
    let mut key_file = std::fs::File::create(&diem_root_key_path).unwrap();
    key_file.write_all(&serialized_keys).unwrap();

    // Prepare log file since we cannot automatically route logs to stderr
    let mut log_file = config_path.clone();
    log_file.push("validator.log");

    // Build a waypoint file so that clients / docker can grab it easily
    let waypoint_file_path = config_path.join("waypoint.txt");
    std::io::Write::write_all(
        &mut std::fs::File::create(&waypoint_file_path).unwrap(),
        genesis_waypoint.to_string().as_bytes(),
    )
    .unwrap();

    // Intentionally leave out instructions on how to connect with different applications
    println!("Completed generating configuration:");
    println!("\tLog file: {:?}", log_file);
    println!("\tConfig path: {:?}", validators[0].config_path());
    println!("\tDiem root key path: {:?}", diem_root_key_path);
    println!("\tWaypoint: {}", genesis_waypoint);
    println!("\tChainId: {}", ChainId::test());

    print_api_config(&validators[0].config, lazy);

    println!("Diem is running, press ctrl-c to exit");
    println!();

    start(&validators[0].config, Some(log_file))
}

pub fn print_api_config(config: &NodeConfig, lazy: bool) {
    println!("\tJSON-RPC endpoint: {}", config.json_rpc.address);
    println!("\tREST API endpoint: {}", config.api.address);
    println!("\tStream-RPC enabled!");

    println!(
        "\tFullNode network: {}",
        config.full_node_networks[0].listen_address
    );
    println!();
    if lazy {
        println!("\tLazy mode is enabled");
        println!();
    }
}

// Fetch chain ID from on-chain resource
fn fetch_chain_id(db: &DbReaderWriter) -> ChainId {
    let blob = db
        .reader
        .get_account_state_with_proof_by_version(
            diem_root_address(),
            (&*db.reader)
                .fetch_synced_version()
                .expect("[diem-node] failed fetching synced version."),
        )
        .expect("[diem-node] failed to get Diem root address account state")
        .0
        .expect("[diem-node] missing Diem root address account state");
    AccountState::try_from(&blob)
        .expect("[diem-node] failed to convert blob to account state")
        .get_chain_id_resource()
        .expect("[diem-node] failed to get chain ID resource")
        .expect("[diem-node] missing chain ID resource")
        .chain_id()
}

fn setup_chunk_executor(db: DbReaderWriter) -> Box<dyn ChunkExecutor> {
    Box::new(Executor::<DpnProto, DiemVM>::new(db))
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

    NodeDebugService::new(addr, logger, config)
}

fn setup_storage_service_servers(
    config: StorageServiceConfig,
    network_handles: Vec<StorageServiceNetworkEvents>,
    db_rw: &DbReaderWriter,
) -> Runtime {
    // For now, spawn all of the storage-service servers on the same runtime.
    let rt = Builder::new_multi_thread()
        .thread_name("storage-service-servers")
        .enable_all()
        .build()
        .expect("Failed to start the DiemNet storage-service runtime.");
    let storage_reader = StorageReader::new(Arc::clone(&db_rw.reader));
    for events in network_handles {
        let service =
            StorageServiceServer::new(config, rt.handle().clone(), storage_reader.clone(), events);
        rt.spawn(service.start());
    }
    rt
}

fn setup_diemnet_data_client(
    _runtime_handle: &Handle,
    config: StorageServiceConfig,
    network_handles: HashMap<NetworkId, storage_service_client::StorageServiceNetworkSender>,
    peer_metadata_storage: Arc<PeerMetadataStorage>,
) -> DiemNetDataClient {
    // Combine all storage service client handles
    let network_client = StorageServiceClient::new(
        StorageServiceMultiSender::new(network_handles),
        peer_metadata_storage,
    );
    let (diemnet_data_client, _data_summary_poller) =
        DiemNetDataClient::new(config, TimeService::real(), network_client);
    // TODO(philiphayes): uncomment this when we're ready to start doing e2e tests
    // runtime_handle.spawn(data_summary_poller);
    diemnet_data_client
}

async fn periodic_state_dump(node_config: NodeConfig, db: DbReaderWriter) {
    use futures::stream::StreamExt;

    let args: Vec<String> = ::std::env::args().collect();

    // Once an hour
    let mut config_interval = IntervalStream::new(tokio::time::interval(
        std::time::Duration::from_secs(60 * 60),
    ))
    .fuse();
    // Once a minute
    let mut version_interval =
        IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(60))).fuse();

    info!("periodic_state_dump task started");

    loop {
        futures::select! {
            _ = config_interval.select_next_some() => {
                info!(config = node_config, args = args, "config and command line arguments");
            }
            _ = version_interval.select_next_some() => {
                let chain_id = fetch_chain_id(&db);
                let ledger_info = if let Ok(ledger_info) = db.reader.get_latest_ledger_info() {
                    ledger_info
                } else {
                    warn!("unable to query latest ledger info");
                    continue;
                };

                let latest_ledger_verion = ledger_info.ledger_info().version();
                let root_hash = ledger_info.ledger_info().transaction_accumulator_hash();

                info!(
                    chain_id = chain_id,
                    latest_ledger_verion = latest_ledger_verion,
                    root_hash = root_hash,
                    "latest ledger version and its corresponding root hash"
                );
            }
        }
    }
}

pub fn setup_environment(node_config: &NodeConfig, logger: Option<Arc<Logger>>) -> DiemHandle {
    let debug_if = setup_debug_interface(node_config, logger);

    let metrics_port = node_config.debug_interface.metrics_server_port;
    let metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || metric_server::start_server(metric_host, metrics_port, false));
    let public_metrics_port = node_config.debug_interface.public_metrics_server_port;
    let public_metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || {
        metric_server::start_server(public_metric_host, public_metrics_port, true)
    });

    let mut instant = Instant::now();
    let (diem_db, db_rw) = DbReaderWriter::wrap(
        DiemDB::open(
            &node_config.storage.dir(),
            false, /* readonly */
            node_config.storage.prune_window,
            node_config.storage.rocksdb_config,
            node_config.storage.account_count_migration,
        )
        .expect("DB should open."),
    );
    let _simple_storage_service = start_storage_service_with_db(node_config, Arc::clone(&diem_db));
    let backup_service = start_backup_service(
        node_config.storage.backup_service_address,
        Arc::clone(&diem_db),
    );

    let genesis_waypoint = node_config.base.waypoint.genesis_waypoint();
    // if there's genesis txn and waypoint, commit it if the result matches.
    if let Some(genesis) = get_genesis_txn(node_config) {
        maybe_bootstrap::<DiemVM>(&db_rw, genesis, genesis_waypoint)
            .expect("Db-bootstrapper should not fail.");
    } else {
        info!("Genesis txn not provided, it's fine if you don't expect to apply it otherwise please double check config");
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
    let mut storage_service_server_network_handles = vec![];
    let mut storage_service_client_network_handles = HashMap::new();

    // Create an event subscription service so that components can be notified of events and reconfigs
    let mut event_subscription_service = EventSubscriptionService::new(
        ON_CHAIN_CONFIG_REGISTRY,
        Arc::new(RwLock::new(db_rw.clone())),
    );
    let mempool_reconfig_subscription = event_subscription_service
        .subscribe_to_reconfigurations()
        .unwrap();

    // Create a consensus subscription for reconfiguration events (if this node is a validator).
    let consensus_reconfig_subscription = if node_config.base.role.is_validator() {
        Some(
            event_subscription_service
                .subscribe_to_reconfigurations()
                .unwrap(),
        )
    } else {
        None
    };

    // Gather all network configs into a single vector.
    let mut network_configs: Vec<&NetworkConfig> = node_config.full_node_networks.iter().collect();
    if let Some(network_config) = node_config.validator_network.as_ref() {
        network_configs.push(network_config);
    }

    // Instantiate every network and collect the requisite endpoints for state_sync, mempool, and consensus.
    let mut network_ids = HashSet::new();
    network_configs.iter().for_each(|config| {
        let network_id = config.network_id;
        // Guarantee there is only one of this network
        if network_ids.contains(&network_id) {
            panic!(
                "Duplicate NetworkId: '{}'.  Can't start node with duplicate networks",
                network_id
            );
        }
        network_ids.insert(network_id);
    });
    let network_ids: Vec<_> = network_ids.into_iter().collect();

    let peer_metadata_storage = PeerMetadataStorage::new(&network_ids);
    for network_config in network_configs.into_iter() {
        debug!("Creating runtime for {}", network_config.network_id);
        let runtime = Builder::new_multi_thread()
            .thread_name(format!("network-{}", network_config.network_id))
            .enable_all()
            .build()
            .expect("Failed to start runtime. Won't be able to start networking.");

        // Entering here gives us a runtime to instantiate all the pieces of the builder
        let _enter = runtime.enter();

        // Perform common instantiation steps
        let mut network_builder = NetworkBuilder::create(
            chain_id,
            node_config.base.role,
            network_config,
            TimeService::real(),
            Some(&mut event_subscription_service),
            peer_metadata_storage.clone(),
        );
        let network_id = network_config.network_id;

        // Create the endpoints to connect the Network to State Sync.
        let (state_sync_sender, state_sync_events) =
            network_builder.add_p2p_service(&state_sync_v1_network_config());
        state_sync_network_handles.push((network_id, state_sync_sender, state_sync_events));

        // TODO(philiphayes): configure which networks we serve the storage service
        // on? for example, if we're a light node we wouldn't want to provide the
        // storage service at all.

        // Register the network-facing storage service with Network.
        let storage_service_events = network_builder
            .add_service(&storage_service_server::network::network_endpoint_config());
        storage_service_server_network_handles.push(storage_service_events);

        // Register the storage-service clients with Network
        let storage_service_sender =
            network_builder.add_client(&storage_service_client::network_endpoint_config());
        storage_service_client_network_handles.insert(network_id, storage_service_sender);

        // Create the endpoints to connect the Network to mempool.
        let (mempool_sender, mempool_events) = network_builder.add_p2p_service(
            &diem_mempool::network::network_endpoint_config(MEMPOOL_NETWORK_CHANNEL_BUFFER_SIZE),
        );
        mempool_network_handles.push((network_id, mempool_sender, mempool_events));

        // Perform steps relevant specifically to Validator networks.
        if network_id.is_validator_network() {
            // A valid config is allowed to have at most one ValidatorNetwork
            // TODO:  `expect_none` would be perfect here, once it is stable.
            if consensus_network_handles.is_some() {
                panic!("There can be at most one validator network!");
            }

            consensus_network_handles = Some(
                network_builder
                    .add_p2p_service(&consensus::network_interface::network_endpoint_config()),
            );
        }

        let network_context = network_builder.network_context();
        network_builder.build(runtime.handle().clone());
        network_builder.start();
        debug!("Network built for network context: {}", network_context);
        network_runtimes.push(runtime);
    }

    // TODO set up on-chain discovery network based on UpstreamConfig.fallback_network
    // and pass network handles to mempool/state sync

    let storage_service_rt = setup_storage_service_servers(
        StorageServiceConfig::default(),
        storage_service_server_network_handles,
        &db_rw,
    );

    let _diemnet_data_client = setup_diemnet_data_client(
        // TODO(philiphayes): probably use state-sync-v2 handle here?
        storage_service_rt.handle(),
        StorageServiceConfig::default(),
        storage_service_client_network_handles,
        peer_metadata_storage.clone(),
    );

    // For state sync to send notifications to mempool and receive notifications from consensus.
    let (mempool_notifier, mempool_listener) =
        mempool_notifications::new_mempool_notifier_listener_pair();
    let (consensus_notifier, consensus_listener) =
        consensus_notifications::new_consensus_notifier_listener_pair(
            node_config.state_sync.client_commit_timeout_ms,
        );

    // Create the state sync multiplexer
    let state_sync = StateSyncMultiplexer::new(
        state_sync_network_handles,
        mempool_notifier,
        consensus_listener,
        Arc::clone(&db_rw.reader),
        chunk_executor,
        node_config,
        genesis_waypoint,
        event_subscription_service,
    );
    let (mp_client_sender, mp_client_events) = channel(AC_SMP_CHANNEL_BUFFER_SIZE);

    let api_runtime = if node_config.api.enabled {
        // bootstrap_api bootstraps a web-server serves for both REST and JSON-RPC API
        bootstrap_api(node_config, chain_id, diem_db, mp_client_sender).unwrap()
    } else {
        bootstrap_rpc(node_config, chain_id, diem_db, mp_client_sender)
    };

    let mut consensus_runtime = None;
    let (consensus_to_mempool_sender, consensus_requests) = channel(INTRA_NODE_CHANNEL_BUFFER_SIZE);

    instant = Instant::now();
    let mempool = diem_mempool::bootstrap(
        node_config,
        Arc::clone(&db_rw.reader),
        mempool_network_handles,
        mp_client_events,
        consensus_requests,
        mempool_listener,
        mempool_reconfig_subscription,
        peer_metadata_storage.clone(),
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
        debug!("Wait until state sync is initialized");
        state_sync.block_until_initialized();
        debug!("State sync initialization complete.");

        // Initialize and start consensus.
        instant = Instant::now();
        consensus_runtime = Some(start_consensus(
            node_config,
            consensus_network_sender,
            consensus_network_events,
            Arc::new(consensus_notifier),
            consensus_to_mempool_sender,
            db_rw.clone(),
            consensus_reconfig_subscription
                .expect("Consensus requires a reconfiguration subscription!"),
            peer_metadata_storage,
        ));
        debug!("Consensus started in {} ms", instant.elapsed().as_millis());
    }

    // Spawn a task which will periodically dump some interesting state
    debug_if
        .runtime()
        .handle()
        .spawn(periodic_state_dump(node_config.to_owned(), db_rw));

    DiemHandle {
        _api: api_runtime,
        _backup: backup_service,
        _consensus_runtime: consensus_runtime,
        _debug: debug_if,
        _mempool: mempool,
        _network_runtimes: network_runtimes,
        _state_sync: state_sync,
        _storage_service: storage_service_rt,
    }
}
