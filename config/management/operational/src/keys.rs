// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_management::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use serde::Serialize;
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct ExtractKey {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The keys name in storage
    #[structopt(long)]
    key_name: String,
    /// Location to store the key
    #[structopt(long)]
    key_file: PathBuf,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

#[derive(Debug, StructOpt)]
pub struct ExtractPublicKey {
    #[structopt(flatten)]
    extract_key: ExtractKey,
}

impl ExtractPublicKey {
    pub fn execute(self) -> Result<(), Error> {
        let config =
            self.extract_key.config.load()?.override_validator_backend(
                &self.extract_key.validator_backend.validator_backend,
            )?;

        let storage = config.validator_backend();
        let key_name = Box::leak(self.extract_key.key_name.into_boxed_str());
        let key = storage.ed25519_public_from_private(key_name)?;
        save_key(&key, key_name, &self.extract_key.key_file)
    }
}

#[derive(Debug, StructOpt)]
pub struct ExtractPrivateKey {
    #[structopt(flatten)]
    extract_key: ExtractKey,
}

impl ExtractPrivateKey {
    pub fn execute(self) -> Result<(), Error> {
        let config =
            self.extract_key.config.load()?.override_validator_backend(
                &self.extract_key.validator_backend.validator_backend,
            )?;

        let storage = config.validator_backend();
        let key_name = Box::leak(self.extract_key.key_name.into_boxed_str());
        let key = storage.ed25519_private(key_name)?;
        save_key(&key, key_name, &self.extract_key.key_file)
    }
}

fn save_key<T: Serialize>(key: &T, key_name: &'static str, path: &PathBuf) -> Result<(), Error> {
    let encoded = lcs::to_bytes(key).map_err(|e| Error::LCS(key_name.to_string(), e))?;
    let mut file = File::create(path).map_err(|e| Error::IO(key_name.to_string(), e))?;
    file.write_all(&encoded)
        .map_err(|e| Error::IO(key_name.to_string(), e))?;
    Ok(())
}
