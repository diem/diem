// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::{Error, LoggerConfig, PersistableConfig, SecureBackend};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_JSON_RPC_ENDPOINT: &str = "https://127.0.0.1:8080";

// Timing related defaults
const DEFAULT_ROTATION_PERIOD_SECS: u64 = 604_800; // 1 week
const DEFAULT_SLEEP_PERIOD_SECS: u64 = 600; // 10 minutes
const DEFAULT_TXN_EXPIRATION_SECS: u64 = 3600; // 1 hour

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyManagerConfig {
    pub logger: LoggerConfig,
    pub json_rpc_endpoint: String,
    pub rotation_period_secs: u64,
    pub secure_backend: SecureBackend,
    pub sleep_period_secs: u64,
    pub txn_expiration_secs: u64,
}

impl Default for KeyManagerConfig {
    fn default() -> KeyManagerConfig {
        KeyManagerConfig {
            json_rpc_endpoint: DEFAULT_JSON_RPC_ENDPOINT.into(),
            logger: LoggerConfig::default(),
            rotation_period_secs: DEFAULT_ROTATION_PERIOD_SECS,
            secure_backend: SecureBackend::InMemoryStorage,
            sleep_period_secs: DEFAULT_SLEEP_PERIOD_SECS,
            txn_expiration_secs: DEFAULT_TXN_EXPIRATION_SECS,
        }
    }
}

impl KeyManagerConfig {
    /// Reads the key manager config file from the given input_path. Paths used in the config are
    /// either absolute or relative to the config location
    pub fn load<P: AsRef<Path>>(input_path: P) -> Result<Self, Error> {
        Self::load_config(&input_path)
    }

    /// Saves the key manager config file to the given output_path.
    pub fn save<P: AsRef<Path>>(&mut self, output_path: P) -> Result<(), Error> {
        self.save_config(&output_path)
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        if let SecureBackend::OnDiskStorage(backend) = &mut self.secure_backend {
            backend.set_data_dir(data_dir);
        }
    }
}
