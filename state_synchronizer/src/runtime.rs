// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{proto::state_synchronizer_grpc, state_sync_service::StateSyncService};
use config::config::{NodeConfig, RoleType};
use grpc_helpers::ServerHandle;
use grpcio::EnvBuilder;
use std::sync::Arc;

/// Handle for State Synchronizer Runtime
pub struct StateSyncRuntime {
    /// gRPC server to serve commit requests from Consensus
    pub grpc_server: Option<ServerHandle>,
}

impl StateSyncRuntime {
    pub fn bootstrap(config: &NodeConfig) -> Self {
        let grpc_server = match config.base.get_role() {
            RoleType::Validator => {
                let env = Arc::new(EnvBuilder::new().name_prefix("grpc-state-sync-").build());
                let handle = StateSyncService;
                let service = state_synchronizer_grpc::create_state_synchronizer(handle);
                let grpc_server = ::grpcio::ServerBuilder::new(env)
                    .register_service(service)
                    .bind(
                        config.state_sync.address.clone(),
                        config.state_sync.service_port,
                    )
                    .build()
                    .expect("[state sync] unable to create grpc server");
                Some(ServerHandle::setup(grpc_server))
            }
            RoleType::FullNode => None,
        };
        Self { grpc_server }
    }
}
