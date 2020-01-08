// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::{path::PathBuf, net::SocketAddr};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SafetyRulesConfig {
    pub backend: SafetyRulesBackend,
    pub service: SafetyRulesService,
}

impl Default for SafetyRulesConfig {
    fn default() -> Self {
        Self {
            backend: SafetyRulesBackend::InMemoryStorage,
            service: SafetyRulesService::Local,
        }
    }
}

impl SafetyRulesConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        if let SafetyRulesBackend::OnDiskStorage(backend) = &mut self.backend {
            backend.set_data_dir(data_dir);
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SafetyRulesBackend {
    InMemoryStorage,
    OnDiskStorage(OnDiskStorageConfig),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OnDiskStorageConfig {
    // In testing scenarios this implies that the default state is okay if
    // a state is not specified.
    pub default: bool,
    // Required path for on disk storage
    pub path: PathBuf,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl Default for OnDiskStorageConfig {
    fn default() -> Self {
        Self {
            default: false,
            path: PathBuf::from("safety_rules.toml"),
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

/// Defines how safety rules should be executed
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SafetyRulesService {
    /// This runs safety rules in the same thread as event processor
    Local,
    /// This is the production, separate service approach
    Process(RemoteService),
    /// This runs safety rules in the same thread as event processor but data is passed through the
    /// light weight RPC (serializer)
    Serializer,
    /// This instructs Consensus that this is an test model, where Consensus should take the
    /// existing config, create a new process, and pass it the config
    SpawnedProcess(RemoteService),
    /// This creates a separate thread to run safety rules, it is similar to a fork / exec style
    Thread,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RemoteService {
    pub server_address: SocketAddr,
    pub consensus_type: ConsensusType,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ConsensusType {
    SignedTransactions,
    Bytes,
    Rounds,
}
