// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_proto::proto::admission_control_grpc::{
    create_admission_control, AdmissionControlClient,
};
use admission_control_service::admission_control_service::AdmissionControlService;
use config::config::NodeConfig;
use consensus::consensus_provider::{make_consensus_provider, ConsensusProvider};
use debug_interface::{node_debug_service::NodeDebugService, proto::node_debug_interface_grpc};
use execution_proto::proto::execution_grpc;
use execution_service::ExecutionService;
use grpc_helpers::ServerHandle;
use grpcio::{ChannelBuilder, EnvBuilder, ServerBuilder};
use grpcio_sys;
use logger::prelude::*;
use mempool::{proto::mempool_grpc::MempoolClient, MempoolRuntime};
use metrics::metric_server;
use network::{
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        ConsensusNetworkEvents, ConsensusNetworkSender, MempoolNetworkEvents, MempoolNetworkSender,
        CONSENSUS_DIRECT_SEND_PROTOCOL, CONSENSUS_RPC_PROTOCOL, MEMPOOL_DIRECT_SEND_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use std::{
    cmp::max,
    convert::{TryFrom, TryInto},
    sync::Arc,
    thread,
    time::Instant,
};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use tokio::runtime::{Builder, Runtime};
use types::account_address::AccountAddress as PeerId;
use vm_validator::vm_validator::VMValidator;

pub struct LibraHandle {
    _ac: ServerHandle,
    _mempool: MempoolRuntime,
    _network_runtime: Runtime,
    consensus: Box<dyn ConsensusProvider>,
    _execution: ServerHandle,
    _storage: ServerHandle,
    _debug: ServerHandle,
}

impl Drop for LibraHandle {
    fn drop(&mut self) {
        self.consensus.stop();
    }
}

fn setup_ac(config: &NodeConfig) -> (::grpcio::Server, AdmissionControlClient) {
    let env = Arc::new(
        EnvBuilder::new()
            .name_prefix("grpc-ac-")
            .cq_count(unsafe { max(grpcio_sys::gpr_cpu_num_cores() as usize / 2, 2) })
            .build(),
    );
    let port = config.admission_control.admission_control_service_port;

    // Create mempool client
    let connection_str = format!("localhost:{}", config.mempool.mempool_service_port);
    let env2 = Arc::new(EnvBuilder::new().name_prefix("grpc-ac-mem-").build());
    let mempool_client = Arc::new(MempoolClient::new(
        ChannelBuilder::new(env2).connect(&connection_str),
    ));

    // Create storage read client
    let storage_client: Arc<dyn StorageRead> = Arc::new(StorageReadServiceClient::new(
        Arc::new(EnvBuilder::new().name_prefix("grpc-ac-sto-").build()),
        "localhost",
        config.storage.port,
    ));

    let vm_validator = Arc::new(VMValidator::new(&config, Arc::clone(&storage_client)));

    let handle = AdmissionControlService::new(
        mempool_client,
        storage_client,
        vm_validator,
        config
            .admission_control
            .need_to_check_mempool_before_validation,
    );
    let service = create_admission_control(handle);
    let server = ServerBuilder::new(Arc::clone(&env))
        .register_service(service)
        .bind(config.admission_control.address.clone(), port)
        .build()
        .expect("Unable to create grpc server");

    let connection_str = format!("localhost:{}", port);
    let client = AdmissionControlClient::new(ChannelBuilder::new(env).connect(&connection_str));
    (server, client)
}

fn setup_executor(config: &NodeConfig) -> ::grpcio::Server {
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
    ));

    let handle = ExecutionService::new(storage_read_client, storage_write_client, config);
    let service = execution_grpc::create_execution(handle);
    ::grpcio::ServerBuilder::new(Arc::new(EnvBuilder::new().name_prefix("grpc-exe-").build()))
        .register_service(service)
        .bind(config.execution.address.clone(), config.execution.port)
        .build()
        .expect("Unable to create grpc server")
}

fn setup_debug_interface(config: &NodeConfig) -> ::grpcio::Server {
    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-debug-").build());
    // Start Debug interface
    let debug_service =
        node_debug_interface_grpc::create_node_debug_interface(NodeDebugService::new());
    ::grpcio::ServerBuilder::new(env)
        .register_service(debug_service)
        .bind(
            config.debug_interface.address.clone(),
            config.debug_interface.admission_control_node_debug_port,
        )
        .build()
        .expect("Unable to create grpc server")
}

pub fn setup_network(
    config: &NodeConfig,
) -> (
    (MempoolNetworkSender, MempoolNetworkEvents),
    (ConsensusNetworkSender, ConsensusNetworkEvents),
    Runtime,
) {
    let runtime = Builder::new()
        .name_prefix("network-")
        .build()
        .expect("Failed to start runtime. Won't be able to start networking.");
    let peer_id = PeerId::try_from(config.base.peer_id.clone()).expect("Invalid PeerId");
    let listen_addr = config.network.listen_address.clone();
    let advertised_addr = config.network.advertised_address.clone();
    let trusted_peers = config
        .base
        .trusted_peers
        .get_trusted_network_peers()
        .clone()
        .into_iter()
        .map(|(peer_id, (signing_public_key, identity_public_key))| {
            (
                peer_id,
                NetworkPublicKeys {
                    signing_public_key,
                    identity_public_key,
                },
            )
        })
        .collect();
    let seed_peers = config
        .network
        .seed_peers
        .seed_peers
        .clone()
        .into_iter()
        .map(|(peer_id, addrs)| (peer_id.try_into().expect("Invalid PeerId"), addrs))
        .collect();
    let network_signing_keypair = config.base.peer_keypairs.get_network_signing_keypair();
    let network_identity_keypair = config.base.peer_keypairs.get_network_identity_keypair();
    let (
        (mempool_network_sender, mempool_network_events),
        (consensus_network_sender, consensus_network_events),
        _listen_addr,
    ) = NetworkBuilder::new(runtime.executor(), peer_id, listen_addr)
        .transport(if config.network.enable_encryption_and_authentication {
            TransportType::TcpNoise
        } else {
            TransportType::Tcp
        })
        .advertised_address(advertised_addr)
        .seed_peers(seed_peers)
        .signing_keys(network_signing_keypair)
        .identity_keys(network_identity_keypair)
        .trusted_peers(trusted_peers)
        .discovery_interval_ms(config.network.discovery_interval_ms)
        .connectivity_check_interval_ms(config.network.connectivity_check_interval_ms)
        .consensus_protocols(vec![
            ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL),
            ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
        ])
        .mempool_protocols(vec![ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL)])
        .direct_send_protocols(vec![
            ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
        ])
        .rpc_protocols(vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)])
        .build();

    (
        (mempool_network_sender, mempool_network_events),
        (consensus_network_sender, consensus_network_events),
        runtime,
    )
}

pub fn setup_environment(node_config: &NodeConfig) -> (AdmissionControlClient, LibraHandle) {
    crash_handler::setup_panic_handler();

    let mut instant = Instant::now();
    let storage = start_storage_service(&node_config);
    debug!(
        "Storage service started in {} ms",
        instant.elapsed().as_millis()
    );

    instant = Instant::now();
    let execution = ServerHandle::setup(setup_executor(&node_config));
    debug!(
        "Execution service started in {} ms",
        instant.elapsed().as_millis()
    );

    instant = Instant::now();
    let (
        (mempool_network_sender, mempool_network_events),
        (consensus_network_sender, consensus_network_events),
        network_runtime,
    ) = setup_network(&node_config);
    debug!("Network started in {} ms", instant.elapsed().as_millis());

    instant = Instant::now();
    let (ac_server, ac_client) = setup_ac(&node_config);
    let ac = ServerHandle::setup(ac_server);
    debug!("AC started in {} ms", instant.elapsed().as_millis());

    instant = Instant::now();
    let mempool =
        MempoolRuntime::bootstrap(&node_config, mempool_network_sender, mempool_network_events);
    debug!("Mempool started in {} ms", instant.elapsed().as_millis());

    let debug_if = ServerHandle::setup(setup_debug_interface(&node_config));

    let metrics_port = node_config.debug_interface.metrics_server_port;
    let metric_host = node_config.debug_interface.address.clone();
    thread::spawn(move || metric_server::start_server((metric_host.as_str(), metrics_port)));

    instant = Instant::now();
    let mut consensus_provider = make_consensus_provider(
        &node_config,
        consensus_network_sender,
        consensus_network_events,
    );
    consensus_provider
        .start()
        .expect("Failed to start consensus. Can't proceed.");
    debug!("Consensus started in {} ms", instant.elapsed().as_millis());

    let libra_handle = LibraHandle {
        _ac: ac,
        _mempool: mempool,
        _network_runtime: network_runtime,
        consensus: consensus_provider,
        _execution: execution,
        _storage: storage,
        _debug: debug_if,
    };
    (ac_client, libra_handle)
}
