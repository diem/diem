// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::BaseConfig, utils};
use anyhow::Result;
use libra_types::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

const GENESIS_DEFAULT: &str = "genesis.blob";

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

    pub fn save(&mut self) -> Result<()> {
        if let Some(genesis) = &self.genesis {
            if self.genesis_file_location.as_os_str().is_empty() {
                self.genesis_file_location = PathBuf::from(GENESIS_DEFAULT);
            }
            let mut file = File::create(self.genesis_file_location())?;
            file.write_all(&lcs::to_bytes(&genesis)?)?;
        }
        Ok(())
    }

    pub fn genesis_file_location(&self) -> PathBuf {
        self.base.full_path(&self.genesis_file_location)
    }

    pub fn randomize_ports(&mut self) {
        self.port = utils::get_available_port();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::RoleType;
    use libra_tools::tempdir::TempPath;
    use libra_types::{transaction::Transaction, write_set::WriteSetMut};

    #[test]
    fn test_no_genesis() {
        let (mut config, _path) = generate_config();
        assert_eq!(config.genesis, None);
        let result = config.load();
        assert!(result.is_ok());
        assert_eq!(config.genesis_file_location, PathBuf::new());
    }

    #[test]
    fn test_some_and_load_genesis() {
        let fake_genesis = Transaction::WriteSet(WriteSetMut::new(vec![]).freeze().unwrap());
        let (mut config, _path) = generate_config();
        config.genesis = Some(fake_genesis.clone());
        config.save().expect("Unable to save");
        // Verifies some without path
        assert_eq!(config.genesis_file_location, PathBuf::from(GENESIS_DEFAULT));

        config.genesis = None;
        let result = config.load();
        assert!(result.is_ok());
        assert_eq!(config.genesis, Some(fake_genesis));
    }

    fn generate_config() -> (ExecutionConfig, TempPath) {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        let base_config = BaseConfig::new(temp_dir.path().into(), RoleType::Validator);
        let mut execution_config = ExecutionConfig::default();
        execution_config.base = Arc::new(base_config);
        (execution_config, temp_dir)
    }
}
