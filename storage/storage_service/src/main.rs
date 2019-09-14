// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executable_helpers::helpers::{setup_executable, ARG_CONFIG_PATH, ARG_DISABLE_LOGGING};
use std::thread;

use config::config::NodeConfig;
use debug_interface::{node_debug_service::NodeDebugService, proto::node_debug_interface_grpc};
use failure::prelude::*;
use grpc_helpers::spawn_service_thread;
use logger::prelude::*;

pub struct StorageNode {
    node_config: NodeConfig,
}

impl Drop for StorageNode {
    fn drop(&mut self) {
        info!("Drop StorageNode");
    }
}

impl StorageNode {
    pub fn new(node_config: NodeConfig) -> Self {
        StorageNode { node_config }
    }

    pub fn run(&self) -> Result<()> {
        info!("Starting storage node");

        let _handle = storage_service::start_storage_service(&self.node_config);

        // Start Debug interface
        let debug_service =
            node_debug_interface_grpc::create_node_debug_interface(NodeDebugService::new());
        let _debug_handle = spawn_service_thread(
            debug_service,
            self.node_config.storage.address.clone(),
            self.node_config.debug_interface.storage_node_debug_port,
            "debug_service",
        );

        info!("Started Storage Service");
        loop {
            thread::park();
        }
    }
}

fn main() {
    let (config, _logger, _args) = setup_executable(
        "Libra Storage node".to_string(),
        vec![ARG_CONFIG_PATH, ARG_DISABLE_LOGGING],
    );

    let storage_node = StorageNode::new(config);

    storage_node.run().expect("Unable to run storage node");
}
