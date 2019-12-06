// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use debug_interface::{
    node_debug_service::NodeDebugService,
    proto::node_debug_interface_server::NodeDebugInterfaceServer,
};
use executable_helpers::helpers::setup_executable;
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use std::net::ToSocketAddrs;
use std::{path::PathBuf, thread};
use structopt::StructOpt;

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
        let rt = tokio::runtime::Runtime::new().unwrap();

        let _handle = storage_service::start_storage_service(&self.node_config);

        // Start Debug interface
        let addr = format!(
            "{}:{}",
            self.node_config.storage.address,
            self.node_config.debug_interface.storage_node_debug_port
        )
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

        rt.spawn(
            tonic::transport::Server::builder()
                .add_service(NodeDebugInterfaceServer::new(NodeDebugService::new()))
                .serve(addr),
        );

        info!("Started Storage Service");
        loop {
            thread::park();
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra Storage Service")]
struct Args {
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Path to NodeConfig
    config: Option<PathBuf>,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
}

fn main() {
    let args = Args::from_args();

    let (config, _logger) =
        setup_executable(args.config.as_ref().map(PathBuf::as_path), args.no_logging);

    let storage_node = StorageNode::new(config);

    storage_node.run().expect("Unable to run storage node");
}
