// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    admission_control_service::AdmissionControlService,
    counters,
    upstream_proxy::{process_network_messages, UpstreamProxyData},
};
use admission_control_proto::proto::admission_control::{
    admission_control_server::AdmissionControlServer, SubmitTransactionRequest,
    SubmitTransactionResponse,
};
use anyhow::Result;
use futures::channel::{mpsc, oneshot};
use libra_config::config::{NodeConfig, RoleType};
use libra_mempool::proto::mempool_client::MempoolClientWrapper;
use network::validator_network::{AdmissionControlNetworkEvents, AdmissionControlNetworkSender};
use std::{collections::HashMap, net::ToSocketAddrs, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient};
use tokio::runtime::{Builder, Runtime};
use vm_validator::vm_validator::VMValidator;

/// Handle for AdmissionControl Runtime
pub struct AdmissionControlRuntime {
    /// gRPC server to serve request between client and AC
    _ac_service_rt: Runtime,
    /// separate AC runtime
    _upstream_proxy: Runtime,
}

impl AdmissionControlRuntime {
    /// setup Admission Control runtime
    pub fn bootstrap(
        config: &NodeConfig,
        network_sender: AdmissionControlNetworkSender,
        network_events: Vec<AdmissionControlNetworkEvents>,
        ac_sender: mpsc::Sender<(
            SubmitTransactionRequest,
            oneshot::Sender<Result<SubmitTransactionResponse>>,
        )>,
    ) -> Self {
        let mut ac_service_rt = Builder::new()
            .thread_name("ac-service-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[admission control] failed to create runtime");

        let port = config.admission_control.admission_control_service_port;

        // Create mempool client if the node is validator.
        let mempool_client = if config.base.role == RoleType::Validator {
            Some(MempoolClientWrapper::new(
                "localhost",
                config.mempool.mempool_service_port,
            ))
        } else {
            None
        };

        // Create storage read client
        let storage_client: Arc<dyn StorageRead> = Arc::new(StorageReadServiceClient::new(
            "localhost",
            config.storage.port,
            &mut ac_service_rt,
        ));

        let admission_control_service =
            AdmissionControlService::new(ac_sender, Arc::clone(&storage_client));

        let vm_validator = Arc::new(VMValidator::new(
            &config,
            Arc::clone(&storage_client),
            ac_service_rt.handle().clone(),
        ));

        let addr = format!("{}:{}", config.admission_control.address, port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        ac_service_rt.spawn(
            tonic::transport::Server::builder()
                .add_service(AdmissionControlServer::new(admission_control_service))
                .serve(addr),
        );

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
        ));

        Self {
            _ac_service_rt: ac_service_rt,
            _upstream_proxy: upstream_proxy_runtime,
        }
    }
}
