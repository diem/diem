// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::builder::GenesisBuilder;
use diem_management::{config::ConfigPath, error::Error, secure_backend::SharedBackend};
use diem_secure_storage::Storage;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MoveModules {
    pub modules: Vec<PathBuf>,
}

impl MoveModules {
    /*pub fn from_disk<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(&path).map_err(|e| Error::UnexpectedError(e.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::UnexpectedError(e.to_string()))?;
        Self::parse(&contents)
    }

    pub fn parse(contents: &str) -> Result<Self, Error> {
        toml::from_str(&contents).map_err(|e| Error::UnexpectedError(e.to_string()))
    }

    pub fn to_toml(&self) -> Result<String, Error> {
        toml::to_string(&self).map_err(|e| Error::UnexpectedError(e.to_string()))
    }*/
}

impl std::fmt::Display for MoveModules {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        //write!(f, "{}", toml::to_string(self).unwrap())
        unimplemented!()
    }
}

#[derive(Debug, StructOpt)]
pub struct SetMoveModules {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(long)]
    path: PathBuf,
    #[structopt(flatten)]
    backend: SharedBackend,
}

impl SetMoveModules {
    pub fn execute(self) -> Result<MoveModules, Error> {
        /*let layout = Layout::from_disk(&self.path)?;

        let config = self
            .config
            .load()?
            .override_shared_backend(&self.backend.shared_backend)?;

        // In order to not break cli compatibility we need to clear the namespace set via cli since
        // it was previously ignored.
        let mut shared_backend = config.shared_backend;
        shared_backend.clear_namespace();

        let storage = Storage::from(&shared_backend);
        GenesisBuilder::new(storage)
            .set_layout(&layout)
            .map_err(|e| Error::StorageWriteError("shared", "layout", e.to_string()))?;

        Ok(layout)*/
        unimplemented!()
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        let contents = "\
            operators = [\"alice\", \"bob\"]\n\
            owners = [\"carol\"]\n\
            diem_root = \"dave\"\n\
            treasury_compliance = \"other_dave\"\n\
        ";

        let layout = Layout::parse(contents).unwrap();
        assert_eq!(
            layout.operators,
            vec!["alice".to_string(), "bob".to_string()]
        );
        assert_eq!(layout.owners, vec!["carol".to_string()]);
        assert_eq!(layout.diem_root, "dave");
        assert_eq!(layout.treasury_compliance, "other_dave");
    }
}*/
