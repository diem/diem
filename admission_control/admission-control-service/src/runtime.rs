// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    admission_control_service::AdmissionControlService,
    counters,
    upstream_proxy::{process_network_messages, UpstreamProxyData},
};
use admission_control_proto::proto::admission_control::create_admission_control;
use block_storage_client::make_block_storage_client;
use futures::channel::mpsc;
use grpc_helpers::ServerHandle;
use grpcio::{ChannelBuilder, EnvBuilder, ServerBuilder};
use libra_config::config::{NodeConfig, RoleType};
use libra_mempool::proto::mempool::MempoolClient;
use network::validator_network::{AdmissionControlNetworkEvents, AdmissionControlNetworkSender};
use std::{cmp::min, collections::HashMap, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient};
use tokio::runtime::{Builder, Runtime};
use vm_validator::vm_validator::VMValidator;

/// Handle for AdmissionControl Runtime
pub struct AdmissionControlRuntime {
    /// gRPC server to serve request between client and AC
    _grpc_server: ServerHandle,
    /// separate AC runtime
    _upstream_proxy: Runtime,
}

impl AdmissionControlRuntime {
    /// setup Admission Control runtime
    pub fn bootstrap(
        config: &NodeConfig,
        network_sender: AdmissionControlNetworkSender,
        network_events: Vec<AdmissionControlNetworkEvents>,
    ) -> Self {
        let (ac_sender, ac_receiver) = mpsc::channel(1_024);

        let env = Arc::new(
            EnvBuilder::new()
                .name_prefix("grpc-ac-")
                .cq_count(min(num_cpus::get() * 2, 32))
                .build(),
        );
        let port = config.admission_control.admission_control_service_port;

        // Create mempool client if the node is validator.
        let connection_str = format!("localhost:{}", config.mempool.mempool_service_port);
        let env2 = Arc::new(EnvBuilder::new().name_prefix("grpc-ac-mem-").build());
        let mempool_client = if config.base.role == RoleType::Validator {
            Some(Arc::new(MempoolClient::new(
                ChannelBuilder::new(env2).connect(&connection_str),
            )))
        } else {
            None
        };

        // Create storage read client
        let storage_client: Arc<dyn StorageRead> = Arc::new(StorageReadServiceClient::new(
            Arc::new(EnvBuilder::new().name_prefix("grpc-ac-sto-").build()),
            "localhost",
            config.storage.port,
        ));

        let block_storage_client = make_block_storage_client(
            config.consensus.consensus_rpc_address.as_str(),
            config.consensus.consensus_rpc_port,
            None,
        );
        let admission_control_service = AdmissionControlService::new(
            ac_sender,
            Arc::clone(&storage_client),
            Arc::new(block_storage_client),
        );

        let vm_validator = Arc::new(VMValidator::new(&config, Arc::clone(&storage_client)));

        let service = create_admission_control(admission_control_service);
        let server = ServerBuilder::new(Arc::clone(&env))
            .register_service(service)
            .bind(config.admission_control.address.clone(), port)
            .build()
            .expect("Unable to create grpc server");

        let upstream_proxy_runtime = Builder::new()
            .thread_name("ac-upstream-proxy-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[admission control] failed to create runtime");

        let executor = upstream_proxy_runtime.handle();

        let upstream_peer_ids = &config.state_sync.upstream_peers.upstream_peers;
        let peer_info: HashMap<_, _> = upstream_peer_ids
            .iter()
            .map(|peer_id| (*peer_id, true))
            .collect();
        counters::UPSTREAM_PEERS.set(upstream_peer_ids.len() as i64);

        let upstream_proxy_data = UpstreamProxyData::new(
            config.admission_control.clone(),
            network_sender,
            config.base.role,
            mempool_client,
            storage_client,
            vm_validator,
            config
                .admission_control
                .need_to_check_mempool_before_validation,
        );
        executor.spawn(process_network_messages(
            upstream_proxy_data,
            network_events,
            peer_info,
            executor.clone(),
            ac_receiver,
        ));

        Self {
            _grpc_server: ServerHandle::setup(server),
            _upstream_proxy: upstream_proxy_runtime,
        }
    }
}
