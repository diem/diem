// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::BaseConfig;
use failure::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SafetyRulesConfig {
    pub backend: SafetyRulesBackend,
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for SafetyRulesConfig {
    fn default() -> Self {
        Self {
            backend: SafetyRulesBackend::InMemoryStorage,
            base: Arc::new(BaseConfig::default()),
        }
    }
}

impl SafetyRulesConfig {
    pub fn load(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        if let SafetyRulesBackend::OnDiskStorage(backend) = &mut self.backend {
            backend.load(base)?;
        }
        Ok(())
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
    pub base: Arc<BaseConfig>,
}

impl OnDiskStorageConfig {
    pub fn load(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        self.base = base;
        Ok(())
    }

    pub fn path(&self) -> PathBuf {
        self.base.full_path(&self.path)
    }
}
