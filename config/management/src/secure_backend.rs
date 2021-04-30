// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use diem_config::config::{self, GitHubConfig, OnDiskStorageConfig, Token, VaultConfig};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    path::PathBuf,
    str::FromStr,
};
use structopt::StructOpt;

pub const BACKEND: &str = "backend";
pub const DISK: &str = "disk";
pub const GITHUB: &str = "github";
pub const MEMORY: &str = "memory";
pub const VAULT: &str = "vault";

// Custom timeouts for vault backend operations when using the management tooling.
const CONNECTION_TIMEOUT_MS: u64 = 10_000;
const RESPONSE_TIMEOUT_MS: u64 = 10_000;

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
            .remove(BACKEND)
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
            GITHUB => {
                let repository_owner = self
                    .parameters
                    .remove("repository_owner")
                    .ok_or_else(|| Error::BackendParsingError("missing repository owner".into()))?;
                let repository = self
                    .parameters
                    .remove("repository")
                    .ok_or_else(|| Error::BackendParsingError("missing repository".into()))?;
                let branch = self.parameters.remove("branch");
                let token = self
                    .parameters
                    .remove("token")
                    .ok_or_else(|| Error::BackendParsingError("missing token".into()))?;
                config::SecureBackend::GitHub(GitHubConfig {
                    namespace: self.parameters.remove("namespace"),
                    repository_owner,
                    repository,
                    branch,
                    token: Token::FromDisk(PathBuf::from(token)),
                })
            }
            MEMORY => config::SecureBackend::InMemoryStorage,
            VAULT => {
                let certificate = self.parameters.remove("ca_certificate").map(PathBuf::from);
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
                    ca_certificate: certificate,
                    token: Token::FromDisk(PathBuf::from(token)),
                    renew_ttl_secs: None,
                    disable_cas: Some(true),
                    connection_timeout_ms: Some(CONNECTION_TIMEOUT_MS),
                    response_timeout_ms: Some(RESPONSE_TIMEOUT_MS),
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

#[macro_export]
macro_rules! secure_backend {
    ($struct_name:ident, $field_name:ident, $purpose:literal) => {
        secure_backend!($struct_name, $field_name, $purpose, "ignore");
    };
    ($struct_name:ident, $field_name:ident, $purpose:literal, $required:literal) => {
        #[derive(Clone, Debug, StructOpt)]
        pub struct $struct_name {
            #[structopt(long,
                help = concat!("Backend for ", $purpose),
                required_unless_one(&["config", $required]),
                long_help = concat!("Backend for ", $purpose, r#"

Secure backends are represented as a semi-colon deliminted key value
pair: "k0=v0;k1=v1;...".  The current supported formats are:
    Vault: "backend=vault;server=URL;token=PATH_TO_TOKEN"
        an optional namespace: "namespace=NAMESPACE"
        an optional server certificate: "ca_certificate=PATH_TO_CERT"
    GitHub: "backend=github;repository_owner=REPOSITORY_OWNER;repository=REPOSITORY;token=PATH_TO_TOKEN"
        an optional branch: "branch=BRANCH", defaults to master
        an optional namespace: "namespace=NAMESPACE"
    InMemory: "backend=memory"
    OnDisk: "backend=disk;path=LOCAL_PATH"
                "#)
            )]
            pub $field_name: Option<SecureBackend>,
        }

        impl FromStr for $struct_name {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let secure_backend = SecureBackend::try_from(s)?;
                let $field_name = $struct_name {
                    $field_name: Some(secure_backend),
                };
                Ok($field_name)
            }
        }
    };
}

secure_backend!(
    ValidatorBackend,
    validator_backend,
    "validator configuration"
);

secure_backend!(SharedBackend, shared_backend, "shared information");

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
        let path = diem_temppath::TempPath::new();
        path.create_as_file().unwrap();
        let disk = format!("backend=disk;path={}", path.path().to_str().unwrap());
        storage(&disk).unwrap();

        let disk = "backend=disk";
        assert!(storage(disk).is_err());
    }

    #[test]
    fn test_github() {
        let path = diem_temppath::TempPath::new();
        path.create_as_file().unwrap();
        let mut file = File::create(path.path()).unwrap();
        file.write_all(b"disk_token").unwrap();
        let path_str = path.path().to_str().unwrap();

        let github = format!(
            "backend=github;repository_owner=diem;repository=diem;token={}",
            path_str
        );
        storage(&github).unwrap();

        let github = format!(
            "backend=github;repository_owner=diem;repository=diem;token={};namespace=test",
            path_str
        );
        storage(&github).unwrap();

        let github = format!(
            "backend=github;repository_owner=diem;repository=diem;branch=genesis;token={};namespace=test",
            path_str
        );
        storage(&github).unwrap();

        let github = "backend=github";

        storage(github).unwrap_err();
    }

    #[test]
    fn test_vault() {
        let path = diem_temppath::TempPath::new();
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
        storage(vault).unwrap_err();
    }

    fn storage(s: &str) -> Result<config::SecureBackend, Error> {
        let management_backend: SecureBackend = s.try_into()?;
        management_backend.try_into()
    }
}
