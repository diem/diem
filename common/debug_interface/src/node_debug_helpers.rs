// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for debug interface.

use crate::proto::node_debug_interface_grpc::NodeDebugInterfaceClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::prelude::*;
use std::{sync::Arc, thread, time};

pub fn create_debug_client(debug_port: u16) -> NodeDebugInterfaceClient {
    let node_connection_str = format!("localhost:{}", debug_port);
    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-debug-").build());
    let ch = ChannelBuilder::new(env).connect(&node_connection_str);
    NodeDebugInterfaceClient::new(ch)
}

pub fn check_node_up(client: &NodeDebugInterfaceClient) {
    let mut attempt = 200;
    let get_details_req = crate::proto::node_debug_interface::GetNodeDetailsRequest::new();

    loop {
        match client.get_node_details(&get_details_req) {
            Ok(_) => {
                info!("Node is up");
                break;
            }
            Err(e) => {
                if attempt > 0 {
                    attempt -= 1;
                    thread::sleep(time::Duration::from_millis(100));
                } else {
                    panic!("Node is not up after many attempts: {}", e);
                }
            }
        }
    }
}
