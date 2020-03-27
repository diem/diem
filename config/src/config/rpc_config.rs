// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RpcConfig {
    pub address: SocketAddr,
}

pub const DEFAULT_JSON_RPC_PORT: u16 = 8080;

impl Default for RpcConfig {
    fn default() -> RpcConfig {
        RpcConfig {
            address: format!("0.0.0.0:{}", DEFAULT_JSON_RPC_PORT)
                .parse()
                .unwrap(),
        }
    }
}

impl RpcConfig {
    pub fn randomize_ports(&mut self) {
        self.address.set_port(utils::get_available_port());
    }
}
