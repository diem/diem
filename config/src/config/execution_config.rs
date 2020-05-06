// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::RootPath;
use anyhow::Result;
use libra_types::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    net::SocketAddr,
    path::PathBuf,
};

const GENESIS_DEFAULT: &str = "genesis.blob";

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutionConfig {
    #[serde(skip)]
    pub genesis: Option<Transaction>,
    pub genesis_file_location: PathBuf,
    pub service: ExecutionCorrectnessService,
}

impl std::fmt::Debug for ExecutionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExecutionConfig {{ genesis: ")?;
        if self.genesis.is_some() {
            write!(f, "Some(...)")?;
        } else {
            write!(f, "None")?;
        }
        write!(
            f,
            ", genesis_file_location: {:?} }}",
            self.genesis_file_location
        )?;
        self.service.fmt(f)
    }
}

impl Default for ExecutionConfig {
    fn default() -> ExecutionConfig {
        ExecutionConfig {
            genesis: None,
            genesis_file_location: PathBuf::new(),
            service: ExecutionCorrectnessService::Local,
        }
    }
}

impl ExecutionConfig {
    pub fn load(&mut self, root_dir: &RootPath) -> Result<()> {
        if !self.genesis_file_location.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.genesis_file_location);
            let mut file = File::open(&path)?;
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;
            self.genesis = Some(lcs::from_bytes(&buffer)?);
        }

        Ok(())
    }

    pub fn save(&mut self, root_dir: &RootPath) -> Result<()> {
        if let Some(genesis) = &self.genesis {
            if self.genesis_file_location.as_os_str().is_empty() {
                self.genesis_file_location = PathBuf::from(GENESIS_DEFAULT);
            }
            let path = root_dir.full_path(&self.genesis_file_location);
            let mut file = File::create(&path)?;
            file.write_all(&lcs::to_bytes(&genesis)?)?;
        }
        Ok(())
    }
}

/// Defines how execution correctness should be run
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecutionCorrectnessService {
    /// This runs execution correctness in the same thread as event processor.
    Local,
    /// This is the production, separate service approach
    Process(RemoteExecutionService),
    /// This runs safety rules in the same thread as event processor but data is passed through the
    /// light weight RPC (serializer)
    Serializer,
    /// This instructs Consensus that this is an test model, where Consensus should take the
    /// existing config, create a new process, and pass to it the config
    SpawnedProcess(RemoteExecutionService),
    /// This creates a separate thread to run execution correctness, it is similar to a fork / exec style
    Thread,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RemoteExecutionService {
    pub server_address: SocketAddr,
}

#[cfg(test)]
mod test {
    use super::*;
    use libra_temppath::TempPath;
    use libra_types::{
        transaction::{ChangeSet, Transaction},
        write_set::WriteSetMut,
    };

    #[test]
    fn test_no_genesis() {
        let (mut config, path) = generate_config();
        assert_eq!(config.genesis, None);
        let root_dir = RootPath::new_path(path.path());
        let result = config.load(&root_dir);
        assert!(result.is_ok());
        assert_eq!(config.genesis_file_location, PathBuf::new());
    }

    #[test]
    fn test_some_and_load_genesis() {
        let fake_genesis = Transaction::WaypointWriteSet(ChangeSet::new(
            WriteSetMut::new(vec![]).freeze().unwrap(),
            vec![],
        ));
        let (mut config, path) = generate_config();
        config.genesis = Some(fake_genesis.clone());
        let root_dir = RootPath::new_path(path.path());
        config.save(&root_dir).expect("Unable to save");
        // Verifies some without path
        assert_eq!(config.genesis_file_location, PathBuf::from(GENESIS_DEFAULT));

        config.genesis = None;
        let result = config.load(&root_dir);
        assert!(result.is_ok());
        assert_eq!(config.genesis, Some(fake_genesis));
    }

    fn generate_config() -> (ExecutionConfig, TempPath) {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        let execution_config = ExecutionConfig::default();
        (execution_config, temp_dir)
    }
}
