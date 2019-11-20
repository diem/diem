// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::BaseConfig, utils};
use failure::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub address: String,
    pub port: u16,
    pub dir: PathBuf,
    pub grpc_max_receive_len: Option<i32>,
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for StorageConfig {
    fn default() -> StorageConfig {
        StorageConfig {
            address: "localhost".to_string(),
            port: 6184,
            dir: PathBuf::from("libradb/db"),
            grpc_max_receive_len: Some(100_000_000),
            base: Arc::new(BaseConfig::default()),
        }
    }
}

impl StorageConfig {
    pub fn load(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        self.base = base;
        Ok(())
    }

    pub fn dir(&self) -> PathBuf {
        self.base.full_path(&self.dir)
    }

    pub fn randomize_ports(&mut self) {
        self.port = utils::get_available_port();
    }
}
