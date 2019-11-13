// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    pub address: String,
    pub port: u16,
    pub dir: PathBuf,
    pub grpc_max_receive_len: Option<i32>,
}

impl Default for StorageConfig {
    fn default() -> StorageConfig {
        StorageConfig {
            address: "localhost".to_string(),
            port: 6184,
            dir: PathBuf::from("libradb/db"),
            grpc_max_receive_len: Some(100_000_000),
        }
    }
}
