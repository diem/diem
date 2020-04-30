// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use libra_config::config::{self, OnDiskStorageConfig, VaultConfig};
use libra_secure_storage::Storage;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    path::PathBuf,
    str::FromStr,
};

/// SecureBackend is a parameter that is stored as set of semi-colon separated key/value pairs. The
/// only expected key is backend which defines which of the SecureBackends the parameters refer to.
/// Some backends require parameters others do not, so that requires a conversion into the
/// config::SecureBackend type to parse.
///
/// Example: backend=vault;server=http://127.0.0.1:8080;token=123456
#[derive(Debug)]
pub struct SecureBackend {
    backend: String,
    parameters: HashMap<String, String>,
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
            "disk" => {
                let mut config = OnDiskStorageConfig::default();
                config.set_data_dir(PathBuf::from(""));
                let path = self
                    .parameters
                    .remove("path")
                    .ok_or_else(|| Error::BackendParsingError("missing path".into()))?;
                config.path = PathBuf::from(path);
                config::SecureBackend::OnDiskStorage(config)
            }
            "memory" => config::SecureBackend::InMemoryStorage,
            "vault" => {
                let server = self
                    .parameters
                    .remove("server")
                    .ok_or_else(|| Error::BackendParsingError("missing server".into()))?;
                let token = self
                    .parameters
                    .remove("token")
                    .ok_or_else(|| Error::BackendParsingError("missing token".into()))?;
                config::SecureBackend::Vault(VaultConfig {
                    default: false,
                    namespace: self.parameters.remove("namespace"),
                    server,
                    // TODO(davidiw) Make this a path to a file
                    token,
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

    #[test]
    fn test_memory() {
        let memory = "backend=memory";
        storage(memory).unwrap();

        let memory = "backend=memory;extra=stuff";
        assert!(storage(memory).is_err());
    }

    #[test]
    fn test_disk() {
        let disk = "backend=disk;path=some_path";
        storage(disk).unwrap();

        let disk = "backend=disk";
        assert!(storage(disk).is_err());
    }

    #[test]
    fn test_vault() {
        let vault = "backend=vault;server=http://127.0.0.1:8080;token=123456";
        storage(vault).unwrap();

        let vault = "backend=vault;server=http://127.0.0.1:8080;token=123456;namespace=test";
        storage(vault).unwrap();

        let vault = "backend=vault";
        assert!(storage(vault).is_err());
    }

    fn storage(s: &str) -> Result<Box<dyn Storage>, Error> {
        let management_backend: SecureBackend = s.try_into()?;
        management_backend.try_into()
    }
}
