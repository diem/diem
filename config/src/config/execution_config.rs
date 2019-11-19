// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::BaseConfig;
use failure::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutionConfig {
    pub address: String,
    pub port: u16,
    pub genesis_file_location: PathBuf,
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for ExecutionConfig {
    fn default() -> ExecutionConfig {
        ExecutionConfig {
            address: "localhost".to_string(),
            port: 6183,
            genesis_file_location: PathBuf::from("genesis.blob"),
            base: Arc::new(BaseConfig::default()),
        }
    }
}

impl ExecutionConfig {
    pub fn load(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        self.base = base;
        Ok(())
    }

    pub fn genesis_file_location(&self) -> PathBuf {
        self.base.full_path(&self.genesis_file_location)
    }
}
