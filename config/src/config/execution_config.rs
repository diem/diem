// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::BaseConfig, utils};
use failure::Result;
use libra_types::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutionConfig {
    pub address: String,
    pub port: u16,
    #[serde(skip)]
    pub genesis: Option<Transaction>,
    pub genesis_file_location: PathBuf,
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for ExecutionConfig {
    fn default() -> ExecutionConfig {
        ExecutionConfig {
            address: "localhost".to_string(),
            port: 6183,
            genesis: None,
            genesis_file_location: PathBuf::new(),
            base: Arc::new(BaseConfig::default()),
        }
    }
}

impl ExecutionConfig {
    pub fn prepare(&mut self, base: Arc<BaseConfig>) {
        self.base = base;
    }

    pub fn load(&mut self) -> Result<()> {
        if !self.genesis_file_location.as_os_str().is_empty() {
            let mut file = File::open(&self.genesis_file_location())?;
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;
            // TODO: update to use `Transaction::WriteSet` variant when ready.
            self.genesis = Some(lcs::from_bytes(&buffer)?);
        }

        Ok(())
    }

    pub fn save(&mut self) {
        if let Some(genesis) = &self.genesis {
            if self.genesis_file_location.as_os_str().is_empty() {
                self.genesis_file_location = PathBuf::from("genesis.blob");
            }
            let mut file =
                File::create(self.genesis_file_location()).expect("Unable to create genesis.blob");
            file.write_all(&lcs::to_bytes(&genesis).expect("Unable to serialize genesis"))
                .expect("Unable to write genesis.blob");
        }
    }

    pub fn genesis_file_location(&self) -> PathBuf {
        self.base.full_path(&self.genesis_file_location)
    }

    pub fn randomize_ports(&mut self) {
        self.port = utils::get_available_port();
    }
}
