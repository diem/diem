// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::builder::GenesisBuilder;
use diem_management::{config::ConfigPath, error::Error, secure_backend::SharedBackend};
use diem_secure_storage::Storage;
use diem_types::{
    chain_id::ChainId,
    on_chain_config::{ConsensusConfigV2, OnChainConsensusConfig},
    transaction::Transaction,
};
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;

/// Note, it is implicitly expected that the storage supports
/// a namespace but one has not been set.
#[derive(Debug, StructOpt)]
pub struct Genesis {
    #[structopt(flatten)]
    pub config: ConfigPath,
    #[structopt(long, required_unless("config"))]
    pub chain_id: Option<ChainId>,
    #[structopt(flatten)]
    pub backend: SharedBackend,
    #[structopt(long)]
    pub path: Option<PathBuf>,
}

impl Genesis {
    fn config(&self) -> Result<diem_management::config::Config, Error> {
        self.config
            .load()?
            .override_chain_id(self.chain_id)
            .override_shared_backend(&self.backend.shared_backend)
    }

    pub fn execute(self) -> Result<Transaction, Error> {
        let config = self.config()?;
        let chain_id = config.chain_id;
        let storage = Storage::from(&config.shared_backend);
        let genesis = GenesisBuilder::new(storage)
            .build(
                chain_id,
                None,
                OnChainConsensusConfig::V2(ConsensusConfigV2 {
                    two_chain: true,
                    decoupled_execution: true,
                    back_pressure_limit: 10,
                    exclude_round: 20,
                }),
            )
            .map_err(|e| Error::UnexpectedError(e.to_string()))?;

        if let Some(path) = self.path {
            let mut file = File::create(path).map_err(|e| {
                Error::UnexpectedError(format!("Unable to create genesis file: {}", e.to_string()))
            })?;
            let bytes = bcs::to_bytes(&genesis).map_err(|e| {
                Error::UnexpectedError(format!("Unable to serialize genesis: {}", e.to_string()))
            })?;
            file.write_all(&bytes).map_err(|e| {
                Error::UnexpectedError(format!("Unable to write genesis file: {}", e.to_string()))
            })?;
        }

        Ok(genesis)
    }
}
