// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApiConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub address: SocketAddr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_cert_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_key_path: Option<String>,
}

pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_REQUEST_CONTENT_LENGTH_LIMIT: u64 = 4 * 1024 * 1024; // 4mb

fn default_enabled() -> bool {
    true
}

impl Default for ApiConfig {
    fn default() -> ApiConfig {
        ApiConfig {
            enabled: false,
            address: format!("{}:{}", DEFAULT_ADDRESS, DEFAULT_PORT)
                .parse()
                .unwrap(),
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl ApiConfig {
    pub fn randomize_ports(&mut self) {
        self.address.set_port(utils::get_available_port());
    }

    pub fn content_length_limit(&self) -> u64 {
        DEFAULT_REQUEST_CONTENT_LENGTH_LIMIT
    }
}
