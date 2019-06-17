// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A secret service node upon run creates its own process.
//! It is a remote process running on address read from node_config.secret_service.address and
//! accepts connections on port node_config.secret_service.secret_service_port.
//! The proto/secret_service.proto file shows the requests that the service accepts and the
//! responses that it gives back. For an example on how to run the secret service see main.rs.

use crate::{proto::secret_service_grpc, secret_service_server::SecretServiceServer};
use config::config::NodeConfig;
use debug_interface::{node_debug_service::NodeDebugService, proto::node_debug_interface_grpc};
use failure::prelude::*;
use grpc_helpers::spawn_service_thread;
use logger::prelude::*;
use std::thread;

#[cfg(test)]
#[path = "unit_tests/secret_service_node_test.rs"]
mod secret_service_test;

/// Secret service node is run a separate process and handles the secret keys.
pub struct SecretServiceNode {
    node_config: NodeConfig,
}

impl SecretServiceNode {
    /// Instantiates the service with a config file.
    pub fn new(node_config: NodeConfig) -> Self {
        SecretServiceNode { node_config }
    }

    /// Starts the secret service
    pub fn run(&self) -> Result<()> {
        info!("Starting secret service node");

        let handle = SecretServiceServer::new();
        let service = secret_service_grpc::create_secret_service(handle);
        let _ss_service_handle = spawn_service_thread(
            service,
            self.node_config.secret_service.address.clone(),
            self.node_config.secret_service.secret_service_port,
            "secret_service",
        );

        // Start Debug interface
        let debug_service =
            node_debug_interface_grpc::create_node_debug_interface(NodeDebugService::new());
        let _debug_handle = spawn_service_thread(
            debug_service,
            self.node_config.secret_service.address.clone(),
            self.node_config
                .debug_interface
                .secret_service_node_debug_port,
            "debug_secret_service",
        );

        info!(
            "Started AdmissionControl node on port {}",
            self.node_config.secret_service.secret_service_port
        );

        loop {
            thread::park();
        }
    }
}
