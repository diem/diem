// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct ExecutionConfig {
    pub address: String,
    pub port: u16,
    pub genesis_file_location: PathBuf,
}

impl Default for ExecutionConfig {
    fn default() -> ExecutionConfig {
        ExecutionConfig {
            address: "localhost".to_string(),
            port: 6183,
            genesis_file_location: PathBuf::from("genesis.blob"),
        }
    }
}
