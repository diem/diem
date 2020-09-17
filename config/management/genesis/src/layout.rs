// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_management::{
    config::ConfigPath, constants, error::Error, secure_backend::SharedBackend,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

/// Layout defines the set of roles to identities within genesis. In practice, these identities
/// will map to distinct namespaces where the expected data should be stored in the deterministic
/// location as defined within this tool.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Layout {
    pub operators: Vec<String>,
    pub owners: Vec<String>,
    pub libra_root: String,
    pub treasury_compliance: String,
}

impl Layout {
    pub fn from_disk<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
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
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).unwrap())
    }
}

#[derive(Debug, StructOpt)]
pub struct SetLayout {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(long)]
    path: PathBuf,
    #[structopt(flatten)]
    backend: SharedBackend,
}

impl SetLayout {
    pub fn execute(self) -> Result<Layout, Error> {
        let layout = Layout::from_disk(&self.path)?;
        let data = layout.to_toml()?;

        let config = self
            .config
            .load()?
            .override_shared_backend(&self.backend.shared_backend)?;
        let mut storage = config.shared_backend_with_namespace(constants::COMMON_NS.to_string());
        storage.set(constants::LAYOUT, data)?;

        Ok(layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        let contents = "\
            operators = [\"alice\", \"bob\"]\n\
            owners = [\"carol\"]\n\
            libra_root = \"dave\"\n\
            treasury_compliance = \"other_dave\"\n\
        ";

        let layout = Layout::parse(contents).unwrap();
        assert_eq!(
            layout.operators,
            vec!["alice".to_string(), "bob".to_string()]
        );
        assert_eq!(layout.owners, vec!["carol".to_string()]);
        assert_eq!(layout.libra_root, "dave");
        assert_eq!(layout.treasury_compliance, "other_dave");
    }
}
