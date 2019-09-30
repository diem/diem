// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::CoreMempool, mempool_service::MempoolService, proto::mempool_grpc,
    shared_mempool::start_shared_mempool,
};
use grpcio::EnvBuilder;
use grpcio_sys;
use libra_config::config::NodeConfig;
use libra_grpc_helpers::ServerHandle;
use libra_network::validator_network::{MempoolNetworkEvents, MempoolNetworkSender};
use libra_storage_client::{StorageRead, StorageReadServiceClient};
use libra_vm_validator::vm_validator::VMValidator;
use std::{
    cmp::max,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

/// Handle for Mempool Runtime
pub struct MempoolRuntime {
    /// gRPC server to serve request from AC and Consensus
    pub grpc_server: ServerHandle,
    /// separate shared mempool runtime
    pub shared_mempool: Runtime,
}

impl MempoolRuntime {
    /// setup Mempool runtime
    pub fn bootstrap(
        config: &NodeConfig,
        network_sender: MempoolNetworkSender,
        network_events: MempoolNetworkEvents,
    ) -> Self {
        let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));

        // setup grpc server
        let env = Arc::new(
            EnvBuilder::new()
                .name_prefix("grpc-mempool-")
                .cq_count(unsafe { max(grpcio_sys::gpr_cpu_num_cores() as usize / 2, 2) })
                .build(),
        );
        let handle = MempoolService {
            core_mempool: Arc::clone(&mempool),
        };
        let service = mempool_grpc::create_mempool(handle);
        let grpc_server = ::grpcio::ServerBuilder::new(env)
            .register_service(service)
            .bind(
                config.mempool.address.clone(),
                config.mempool.mempool_service_port,
            )
            .build()
            .expect("[mempool] unable to create grpc server");

        // setup shared mempool
        let storage_client: Arc<dyn StorageRead> = Arc::new(StorageReadServiceClient::new(
            Arc::new(EnvBuilder::new().name_prefix("grpc-mem-sto-").build()),
            "localhost",
            config.storage.port,
        ));
        let vm_validator = Arc::new(VMValidator::new(&config, Arc::clone(&storage_client)));
        let shared_mempool = start_shared_mempool(
            config,
            mempool,
            network_sender,
            network_events,
            storage_client,
            vm_validator,
            vec![],
            None,
        );
        Self {
            grpc_server: ServerHandle::setup(grpc_server),
            shared_mempool,
        }
    }
}
