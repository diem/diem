// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// JSON RPC endpoint related defaults
const DEFAULT_JSON_RPC_ENDPOINT: &str = "https://127.0.0.1:8080";

// Key manager timing related defaults
const DEFAULT_ROTATION_PERIOD_SECS: u64 = 604_800; // 1 week
const DEFAULT_SLEEP_PERIOD_SECS: u64 = 600; // 10 minutes
const DEFAULT_TXN_EXPIRATION_SECS: u64 = 3600; // 1 hour

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SecureConfig {
    pub key_manager: KeyManagerConfig,
}

impl SecureConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.key_manager.set_data_dir(data_dir);
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyManagerConfig {
    pub rotation_period_secs: u64,
    pub sleep_period_secs: u64,
    pub txn_expiration_secs: u64,

    pub json_rpc_endpoint: String,
    pub secure_backend: SecureBackend,
}

impl Default for KeyManagerConfig {
    fn default() -> KeyManagerConfig {
        KeyManagerConfig {
            rotation_period_secs: DEFAULT_ROTATION_PERIOD_SECS,
            sleep_period_secs: DEFAULT_SLEEP_PERIOD_SECS,
            txn_expiration_secs: DEFAULT_TXN_EXPIRATION_SECS,
            json_rpc_endpoint: DEFAULT_JSON_RPC_ENDPOINT.into(),
            secure_backend: SecureBackend::InMemoryStorage,
        }
    }
}

impl KeyManagerConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        if let SecureBackend::OnDiskStorage(backend) = &mut self.secure_backend {
            backend.set_data_dir(data_dir);
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SecureBackend {
    GitHub(GitHubConfig),
    InMemoryStorage,
    Vault(VaultConfig),
    OnDiskStorage(OnDiskStorageConfig),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GitHubConfig {
    /// The owner or account that hosts a repository
    pub owner: String,
    /// The repository where storage will mount
    pub repository: String,
    /// The authorization token for accessing the repository
    pub token: String,
    /// A namespace is an optional portion of the path to a key stored within OnDiskStorage. For
    /// example, a key, S, without a namespace would be available in S, with a namespace, N, it
    /// would be in N/S.
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VaultConfig {
    /// A namespace is an optional portion of the path to a key stored within Vault. For example,
    /// a secret, S, without a namespace would be available in secret/data/S, with a namespace, N, it
    /// would be in secret/data/N/S.
    pub namespace: Option<String>,
    /// Vault's URL, note: only HTTP is currently supported.
    pub server: String,
    /// The authorization token for access secrets
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OnDiskStorageConfig {
    // Required path for on disk storage
    pub path: PathBuf,
    /// A namespace is an optional portion of the path to a key stored within OnDiskStorage. For
    /// example, a key, S, without a namespace would be available in S, with a namespace, N, it
    /// would be in N/S.
    pub namespace: Option<String>,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl Default for OnDiskStorageConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            path: PathBuf::from("secure_storage.toml"),
            data_dir: PathBuf::from("/opt/libra/data/common"),
        }
    }
}

impl OnDiskStorageConfig {
    pub fn path(&self) -> PathBuf {
        if self.path.is_relative() {
            self.data_dir.join(&self.path)
        } else {
            self.path.clone()
        }
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }
}
