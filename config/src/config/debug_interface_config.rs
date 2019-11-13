// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct DebugInterfaceConfig {
    pub admission_control_node_debug_port: u16,
    pub storage_node_debug_port: u16,
    // This has similar use to the core-node-debug-server itself
    pub metrics_server_port: u16,
    pub public_metrics_server_port: u16,
    pub address: String,
}

impl Default for DebugInterfaceConfig {
    fn default() -> DebugInterfaceConfig {
        DebugInterfaceConfig {
            admission_control_node_debug_port: 6191,
            storage_node_debug_port: 6194,
            metrics_server_port: 9101,
            public_metrics_server_port: 9102,
            address: "localhost".to_string(),
        }
    }
}
