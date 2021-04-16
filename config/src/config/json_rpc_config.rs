// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct JsonRpcConfig {
    pub address: SocketAddr,
    pub batch_size_limit: u16,
    pub page_size_limit: u16,
    pub content_length_limit: usize,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    #[serde(default)]
    pub stream_rpc: StreamConfig,
}

pub const DEFAULT_JSON_RPC_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_JSON_RPC_PORT: u16 = 8080;
pub const DEFAULT_BATCH_SIZE_LIMIT: u16 = 20;
pub const DEFAULT_PAGE_SIZE_LIMIT: u16 = 1000;
pub const DEFAULT_CONTENT_LENGTH_LIMIT: usize = 32 * 1024; // 32kb

impl Default for JsonRpcConfig {
    fn default() -> JsonRpcConfig {
        JsonRpcConfig {
            address: format!("{}:{}", DEFAULT_JSON_RPC_ADDRESS, DEFAULT_JSON_RPC_PORT)
                .parse()
                .unwrap(),
            batch_size_limit: DEFAULT_BATCH_SIZE_LIMIT,
            page_size_limit: DEFAULT_PAGE_SIZE_LIMIT,
            content_length_limit: DEFAULT_CONTENT_LENGTH_LIMIT,
            tls_cert_path: None,
            tls_key_path: None,
            stream_rpc: StreamConfig::default(),
        }
    }
}

impl JsonRpcConfig {
    pub fn randomize_ports(&mut self) {
        self.address.set_port(utils::get_available_port());
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StreamConfig {
    pub enabled: bool,
    pub subscription_fetch_size: u64,
    pub send_queue_size: usize,
    pub poll_interval_ms: u64,
}

pub const DEFAULT_STREAM_RPC_SUBSCRIPTION_FETCH_SIZE: u64 = 100;
pub const DEFAULT_STREAM_RPC_SEND_QUEUE_SIZE: usize = 100;
pub const DEFAULT_STREAM_RPC_POLL_INTERVAL_MS: u64 = 1000;

impl Default for StreamConfig {
    fn default() -> StreamConfig {
        StreamConfig {
            enabled: false,
            subscription_fetch_size: DEFAULT_STREAM_RPC_SUBSCRIPTION_FETCH_SIZE,
            send_queue_size: DEFAULT_STREAM_RPC_SEND_QUEUE_SIZE,
            poll_interval_ms: DEFAULT_STREAM_RPC_POLL_INTERVAL_MS,
        }
    }
}
