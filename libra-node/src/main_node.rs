// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_service::runtime::AdmissionControlRuntime;
use config::config::{NetworkConfig, NodeConfig, RoleType};
use consensus::consensus_provider::{make_consensus_provider, ConsensusProvider};
use crypto::{ed25519::*, ValidKey};
use debug_interface::{node_debug_service::NodeDebugService, proto::create_node_debug_interface};
use executor::Executor;
use grpc_helpers::ServerHandle;
use grpcio::EnvBuilder;
use libra_logger::prelude::*;
use libra_mempool::MempoolRuntime;
use libra_types::account_address::AccountAddress as PeerId;
use metrics::metric_server;
use network::{
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        LibraNetworkProvider,
        // when you add a new protocol const, you must add this in either
        // .direct_send_protocols or .rpc_protocols vector of network_builder in setup_network()
        ADMISSION_CONTROL_RPC_PROTOCOL,
        CONSENSUS_DIRECT_SEND_PROTOCOL,
        CONSENSUS_RPC_PROTOCOL,
        MEMPOOL_DIRECT_SEND_PROTOCOL,
        STATE_SYNCHRONIZER_MSG_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use state_synchronizer::StateSynchronizer;
use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
    sync::Arc,
    thread,
    time::Instant,
};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use tokio::runtime::{Builder, Runtime};
use vm_runtime::MoveVM;

pub struct LibraHandle {
    _ac: AdmissionControlRuntime,
    _mempool: Option<MempoolRuntime>,
    _state_synchronizer: StateSynchronizer,
    _network_runtimes: Vec<Runtime>,
    consensus: Option<Box<dyn ConsensusProvider>>,
    _storage: ServerHandle,
    _debug: ServerHandle,
}

impl Drop for LibraHandle {
    fn drop(&mut self) {
        if let Some(consensus) = &mut self.consensus {
            consensus.stop();
        }
    }
}

fn setup_executor(config: &NodeConfig) -> Arc<Executor<MoveVM>> {
    let client_env = Arc::new(EnvBuilder::new().name_prefix("grpc-exe-sto-").build());
    let storage_read_client = Arc::new(StorageReadServiceClient::new(
        Arc::clone(&client_env),
        &config.storage.address,
        config.storage.port,
    ));
    let storage_write_client = Arc::new(StorageWriteServiceClient::new(
        Arc::clone(&client_env),
        &config.storage.address,
        config.storage.port,
        config.storage.grpc_max_receive_len,
    ));

    Arc::new(Executor::new(
        Arc::clone(&storage_read_client) as Arc<dyn StorageRead>,
        storage_write_client,
        config,
    ))
}

fn setup_debug_interface(config: &NodeConfig) -> ::grpcio::Server {
    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-debug-").build());
    // Start Debug interface
    let debug_service = create_node_debug_interface(NodeDebugService::new());
    ::grpcio::ServerBuilder::new(env)
        .register_service(debug_service)
        .bind(
            config.debug_interface.address.clone(),
            config.debug_interface.admission_control_node_debug_port,
        )
        .build()
        .expect("Unable to create grpc server")
}

// TODO(abhayb): Move to network crate (similar to consensus).
pub fn setup_network(
    peer_id: PeerId,
    config: &mut NetworkConfig,
) -> (Runtime, Box<dyn LibraNetworkProvider>) {
    let runtime = Builder::new()
        .name_prefix("network-")
        .build()
        .expect("Failed to start runtime. Won't be able to start networking.");
    let role: RoleType = (&config.role).into();
    let mut network_builder = NetworkBuilder::new(
        runtime.executor(),
        peer_id,
        config.listen_address.clone(),
        role,
    );
    network_builder
        .permissioned(config.is_permissioned)
        .advertised_address(config.advertised_address.clone())
        .direct_send_protocols(vec![
            ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            ProtocolId::from_static(STATE_SYNCHRONIZER_MSG_PROTOCOL),
        ])
        .rpc_protocols(vec![
            ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL),
            ProtocolId::from_static(ADMISSION_CONTROL_RPC_PROTOCOL),
        ]);
    if config.is_permissioned {
        // If the node wants to run in permissioned mode, it should also have authentication and
        // encryption.
        assert!(
            config.enable_encryption_and_authentication,
            "Permissioned network end-points should use authentication"
        );
        let trusted_peers = config
            .network_peers
            .peers
            .iter()
            .map(|(peer_id, keys)| {
                (
                    PeerId::from_str(peer_id).unwrap(),
                    NetworkPublicKeys {
                        signing_public_key: keys.network_signing_pubkey.clone(),
                        identity_public_key: keys.network_identity_pubkey.clone(),
                    },
                )
            })
            .collect();
        let seed_peers = config
            .seed_peers
            .seed_peers
            .clone()
            .into_iter()
            .map(|(peer_id, addrs)| (peer_id.try_into().expect("Invalid PeerId"), addrs))
            .collect();
        let network_signing_private = config.network_keypairs.take_network_signing_private()
            .expect("Failed to move network signing private key out of NodeConfig, key not set or moved already");
        let network_signing_public: Ed25519PublicKey = (&network_signing_private).into();
        network_builder
            .transport(TransportType::TcpNoise(Some(
                config.network_keypairs.get_network_identity_keypair(),
            )))
            .connectivity_check_interval_ms(config.connectivity_check_interval_ms)
            .seed_peers(seed_peers)
            .trusted_peers(trusted_peers)
            .signing_keys((network_signing_private, network_signing_public))
            .discovery_interval_ms(config.discovery_interval_ms);
    } else if config.enable_encryption_and_authentication {
        // Even if a network end-point is permissionless, it might want to prove its identity to
        // another peer it connects to. For this, we use TCP + Noise but in a permission-less way.
        network_builder.transport(TransportType::PermissionlessTcpNoise(Some(
            config.network_keypairs.get_network_identity_keypair(),
        )));
    } else {
        network_builder.transport(TransportType::Tcp);
    }
    let (_listen_addr, network_provider) = network_builder.build();
    (runtime, network_provider)
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
    let storage = start_storage_service(&node_config);
    debug!(
        "Storage service started in {} ms",
        instant.elapsed().as_millis()
    );

    instant = Instant::now();
    let executor = setup_executor(&node_config);
    debug!("Executor setup in {} ms", instant.elapsed().as_millis());
    let mut network_runtimes = vec![];
    let mut state_sync_network_handles = vec![];
    let mut ac_network_sender = None;
    let mut ac_network_events = vec![];
    let mut validator_network_provider = None;

    for i in 0..node_config.networks.len() {
        let peer_id =
            PeerId::try_from(node_config.networks[i].peer_id.clone()).expect("Invalid PeerId");
        let (runtime, mut network_provider) = setup_network(peer_id, &mut node_config.networks[i]);
        state_sync_network_handles.push(network_provider.add_state_synchronizer(vec![
            ProtocolId::from_static(STATE_SYNCHRONIZER_MSG_PROTOCOL),
        ]));

        let (ac_sender, ac_events) =
            network_provider.add_admission_control(vec![ProtocolId::from_static(
                ADMISSION_CONTROL_RPC_PROTOCOL,
            )]);
        ac_network_events.push(ac_events);

        let network = &node_config.networks[i];
        if let RoleType::Validator = (&network.role).into() {
            validator_network_provider = Some((peer_id, runtime, network_provider));
            ac_network_sender = Some(ac_sender);
        } else {
            if node_config.is_upstream_network(network) {
                ac_network_sender = Some(ac_sender);
            }
            // For non-validator roles, the peer_id should be derived from the network identity
            // key.
            assert_eq!(
                peer_id,
                PeerId::try_from(
                    network
                        .network_keypairs
                        .get_network_identity_public()
                        .to_bytes()
                )
                .unwrap()
            );
            // Start the network provider.
            runtime.executor().spawn(network_provider.start());
            network_runtimes.push(runtime);
            debug!("Network started for peer_id: {}", peer_id);
        }
    }

    let debug_if = ServerHandle::setup(setup_debug_interface(&node_config));

    let metrics_port = node_config.debug_interface.metrics_server_port;
    let metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || metric_server::start_server((metric_host.as_str(), metrics_port)));

    let state_synchronizer = StateSynchronizer::bootstrap(
        state_sync_network_handles,
        Arc::clone(&executor),
        &node_config,
    );
    let admission_control = AdmissionControlRuntime::bootstrap(
        &node_config,
        ac_network_sender.unwrap(),
        ac_network_events,
    );

    let mut mempool = None;
    let mut consensus = None;
    if let Some((peer_id, runtime, mut network_provider)) = validator_network_provider {
        // Note: We need to start network provider before consensus, because the consensus
        // initialization is blocked on state synchronizer to sync to the initial root ledger
        // info, which in turn cannot make progress before network initialization
        // because the NewPeer events which state synchronizer uses to know its
        // peers are delivered by network provider. If we were to start network
        // provider after consensus, we create a cyclic dependency from
        // network provider -> consensus -> state synchronizer -> network provider. This deadlock
        // was observed in GitHub Issue #749. A long term fix might be make
        // consensus initialization async instead of blocking on state synchronizer.
        let (mempool_network_sender, mempool_network_events) = network_provider
            .add_mempool(vec![ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL)]);
        let (consensus_network_sender, consensus_network_events) =
            network_provider.add_consensus(vec![
                ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL),
                ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            ]);
        runtime.executor().spawn(network_provider.start());
        network_runtimes.push(runtime);
        debug!("Network started for peer_id: {}", peer_id);

        // Initialize and start mempool.
        instant = Instant::now();
        mempool = Some(MempoolRuntime::bootstrap(
            &node_config,
            mempool_network_sender,
            mempool_network_events,
        ));
        debug!("Mempool started in {} ms", instant.elapsed().as_millis());

        // Initialize and start consensus.
        instant = Instant::now();
        let mut consensus_provider = make_consensus_provider(
            node_config,
            consensus_network_sender,
            consensus_network_events,
            executor,
            state_synchronizer.create_client(),
        );
        consensus_provider
            .start()
            .expect("Failed to start consensus. Can't proceed.");
        consensus = Some(consensus_provider);
        debug!("Consensus started in {} ms", instant.elapsed().as_millis());
    }

    LibraHandle {
        _network_runtimes: network_runtimes,
        _ac: admission_control,
        _mempool: mempool,
        _state_synchronizer: state_synchronizer,
        consensus,
        _storage: storage,
        _debug: debug_if,
    }
}
