// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DebugInterfaceConfig {
    pub admission_control_node_debug_port: u16,
    pub address: String,
    pub diem_trace: DiemTraceConfig,
    pub metrics_server_port: u16,
    pub public_metrics_server_port: u16,
}

impl Default for DebugInterfaceConfig {
    fn default() -> DebugInterfaceConfig {
        DebugInterfaceConfig {
            admission_control_node_debug_port: 6191,
            address: "0.0.0.0".to_string(),
            metrics_server_port: 9101,
            public_metrics_server_port: 9102,
            diem_trace: DiemTraceConfig::default(),
        }
    }
}

impl DebugInterfaceConfig {
    pub fn randomize_ports(&mut self) {
        self.admission_control_node_debug_port = utils::get_available_port();
        self.metrics_server_port = utils::get_available_port();
        self.public_metrics_server_port = utils::get_available_port();
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiemTraceConfig {
    pub sampling: HashMap<String, String>,
}

impl Default for DiemTraceConfig {
    fn default() -> DiemTraceConfig {
        let mut map = HashMap::new();
        map.insert(String::from("txn"), String::from("1/100"));
        map.insert(String::from("block"), String::from("1/1"));
        DiemTraceConfig { sampling: map }
    }
}
