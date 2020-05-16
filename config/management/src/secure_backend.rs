// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use libra_config::config::{self, OnDiskStorageConfig, Token, VaultConfig};
use libra_secure_storage::Storage;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    path::PathBuf,
    str::FromStr,
};

pub const DISK: &str = "disk";
pub const MEMORY: &str = "memory";
pub const VAULT: &str = "vault";

/// SecureBackend is a parameter that is stored as set of semi-colon separated key/value pairs. The
/// only expected key is backend which defines which of the SecureBackends the parameters refer to.
/// Some backends require parameters others do not, so that requires a conversion into the
/// config::SecureBackend type to parse.
///
/// Example: backend=vault;server=http://127.0.0.1:8080;token=/path/to/token
#[derive(Clone, Debug)]
pub struct SecureBackend {
    pub backend: String,
    pub parameters: HashMap<String, String>,
}

impl SecureBackend {
    const BACKEND: &'static str = "backend";
}

impl FromStr for SecureBackend {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SecureBackend::try_from(s)
    }
}

impl TryFrom<&str> for SecureBackend {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Error> {
        let kvs = s.split(';');
        let mut parameters = HashMap::new();
        for pair in kvs {
            let kv = pair.split('=').collect::<Vec<_>>();
            if kv.len() != 2 {
                return Err(Error::BackendInvalidKeyValue(format!("{:?}", kv)));
            }
            parameters.insert(kv[0].into(), kv[1].into());
        }
        let backend = parameters
            .remove(Self::BACKEND)
            .ok_or(Error::BackendMissingBackendKey)?;
        Ok(Self {
            backend,
            parameters,
        })
    }
}

impl TryInto<config::SecureBackend> for SecureBackend {
    type Error = Error;

    fn try_into(mut self) -> Result<config::SecureBackend, Error> {
        let backend = match self.backend.as_ref() {
            DISK => {
                let mut config = OnDiskStorageConfig::default();
                config.set_data_dir(PathBuf::from(""));
                let path = self
                    .parameters
                    .remove("path")
                    .ok_or_else(|| Error::BackendParsingError("missing path".into()))?;
                config.path = PathBuf::from(path);
                config.namespace = self.parameters.remove("namespace");
                config::SecureBackend::OnDiskStorage(config)
            }
            MEMORY => config::SecureBackend::InMemoryStorage,
            VAULT => {
                let server = self
                    .parameters
                    .remove("server")
                    .ok_or_else(|| Error::BackendParsingError("missing server".into()))?;
                let token = self
                    .parameters
                    .remove("token")
                    .ok_or_else(|| Error::BackendParsingError("missing token".into()))?;
                config::SecureBackend::Vault(VaultConfig {
                    namespace: self.parameters.remove("namespace"),
                    server,
                    token: Token::new_disk(PathBuf::from(token)),
                })
            }
            _ => panic!("Invalid backend: {}", self.backend),
        };

        if !self.parameters.is_empty() {
            let error = format!("found extra parameters: {:?}", self.parameters);
            return Err(Error::BackendParsingError(error));
        }

        Ok(backend)
    }
}

impl TryInto<Box<dyn Storage>> for SecureBackend {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn Storage>, Error> {
        let config: config::SecureBackend = self.try_into()?;
        Ok((&config).into())
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};

    #[test]
    fn test_memory() {
        let memory = "backend=memory";
        storage(memory).unwrap();

        let memory = "backend=memory;extra=stuff";
        assert!(storage(memory).is_err());
    }

    #[test]
    fn test_disk() {
        let path = libra_temppath::TempPath::new();
        path.create_as_file().unwrap();
        let disk = format!("backend=disk;path={}", path.path().to_str().unwrap());
        storage(&disk).unwrap();

        let disk = "backend=disk";
        assert!(storage(disk).is_err());
    }

    #[test]
    fn test_vault() {
        let path = libra_temppath::TempPath::new();
        path.create_as_file().unwrap();
        let mut file = File::create(path.path()).unwrap();
        file.write_all(b"disk_token").unwrap();
        let path_str = path.path().to_str().unwrap();

        let vault = format!(
            "backend=vault;server=http://127.0.0.1:8080;token={}",
            path_str
        );
        storage(&vault).unwrap();

        let vault = format!(
            "backend=vault;server=http://127.0.0.1:8080;token={};namespace=test",
            path_str
        );
        storage(&vault).unwrap();

        let vault = "backend=vault";
        assert!(storage(vault).is_err());
    }

    fn storage(s: &str) -> Result<Box<dyn Storage>, Error> {
        let management_backend: SecureBackend = s.try_into()?;
        management_backend.try_into()
    }
}
