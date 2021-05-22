// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::Error;
use diem_secure_storage::{
    GitHubStorage, InMemoryStorage, Namespaced, OnDiskStorage, Storage, VaultStorage,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecureBackend {
    GitHub(GitHubConfig),
    InMemoryStorage,
    Vault(VaultConfig),
    OnDiskStorage(OnDiskStorageConfig),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubConfig {
    /// The owner or account that hosts a repository
    pub repository_owner: String,
    /// The repository where storage will mount
    pub repository: String,
    /// The branch containing storage, defaults to master
    pub branch: Option<String>,
    /// The authorization token for accessing the repository
    pub token: Token,
    /// A namespace is an optional portion of the path to a key stored within GitHubConfig. For
    /// example, a key, S, without a namespace would be available in S, with a namespace, N, it
    /// would be in N/S.
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VaultConfig {
    /// Optional SSL Certificate for the vault host, this is expected to be a full path.
    pub ca_certificate: Option<PathBuf>,
    /// A namespace is an optional portion of the path to a key stored within Vault. For example,
    /// a secret, S, without a namespace would be available in secret/data/S, with a namespace, N, it
    /// would be in secret/data/N/S.
    pub namespace: Option<String>,
    /// Vault leverages leases on many tokens, specify this to automatically have your lease
    /// renewed up to that many seconds more. If this is not specified, the lease will not
    /// automatically be renewed.
    pub renew_ttl_secs: Option<u32>,
    /// Vault's URL, note: only HTTP is currently supported.
    pub server: String,
    /// The authorization token for accessing secrets
    pub token: Token,
    /// Disable check-and-set when writing secrets to Vault
    pub disable_cas: Option<bool>,
    /// Timeout for new vault socket connections, in milliseconds.
    pub connection_timeout_ms: Option<u64>,
    /// Timeout for generic vault operations (e.g., reads and writes), in milliseconds.
    pub response_timeout_ms: Option<u64>,
}

impl VaultConfig {
    pub fn ca_certificate(&self) -> Result<String, Error> {
        let path = self
            .ca_certificate
            .as_ref()
            .ok_or(Error::Missing("ca_certificate"))?;
        read_file(path)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OnDiskStorageConfig {
    // Required path for on disk storage
    pub path: PathBuf,
    /// A namespace is an optional portion of the path to a key stored within OnDiskStorage. For
    /// example, a key, S, without a namespace would be available in S, with a namespace, N, it
    /// would be in N/S.
    pub namespace: Option<String>,
    #[serde(skip)]
    data_dir: PathBuf,
}

/// Tokens can either be directly within this config or stored somewhere on disk.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Token {
    FromConfig(String),
    /// This is an absolute path and not relative to data_dir
    FromDisk(PathBuf),
}

impl Token {
    pub fn read_token(&self) -> Result<String, Error> {
        match self {
            Token::FromDisk(path) => read_file(path),
            Token::FromConfig(token) => Ok(token.clone()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenFromConfig {
    token: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenFromDisk {
    path: PathBuf,
}

impl Default for OnDiskStorageConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            path: PathBuf::from("secure_storage.json"),
            data_dir: PathBuf::from("/opt/diem/data"),
        }
    }
}

impl OnDiskStorageConfig {
    pub fn path(&self) -> PathBuf {
        if self.path.is_relative() {
            self.data_dir.join(&self.path)
        } else {
            self.path.clone()
        }
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }
}

fn read_file(path: &Path) -> Result<String, Error> {
    let mut file =
        File::open(path).map_err(|e| Error::IO(path.to_str().unwrap().to_string(), e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| Error::IO(path.to_str().unwrap().to_string(), e))?;
    Ok(contents)
}

impl From<&SecureBackend> for Storage {
    fn from(backend: &SecureBackend) -> Self {
        match backend {
            SecureBackend::GitHub(config) => {
                let storage = Storage::from(GitHubStorage::new(
                    config.repository_owner.clone(),
                    config.repository.clone(),
                    config
                        .branch
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| "master".to_string()),
                    config.token.read_token().expect("Unable to read token"),
                ));
                if let Some(namespace) = &config.namespace {
                    Storage::from(Namespaced::new(namespace, Box::new(storage)))
                } else {
                    storage
                }
            }
            SecureBackend::InMemoryStorage => Storage::from(InMemoryStorage::new()),
            SecureBackend::OnDiskStorage(config) => {
                let storage = Storage::from(OnDiskStorage::new(config.path()));
                if let Some(namespace) = &config.namespace {
                    Storage::from(Namespaced::new(namespace, Box::new(storage)))
                } else {
                    storage
                }
            }
            SecureBackend::Vault(config) => {
                let storage = Storage::from(VaultStorage::new(
                    config.server.clone(),
                    config.token.read_token().expect("Unable to read token"),
                    config
                        .ca_certificate
                        .as_ref()
                        .map(|_| config.ca_certificate().unwrap()),
                    config.renew_ttl_secs,
                    config.disable_cas.map_or_else(|| true, |disable| !disable),
                    config.connection_timeout_ms,
                    config.response_timeout_ms,
                ));
                if let Some(namespace) = &config.namespace {
                    Storage::from(Namespaced::new(namespace, Box::new(storage)))
                } else {
                    storage
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Config {
        vault: VaultConfig,
    }

    #[test]
    fn test_token_config_parsing() {
        let from_config = Config {
            vault: VaultConfig {
                namespace: None,
                server: "127.0.0.1:8200".to_string(),
                ca_certificate: None,
                token: Token::FromConfig("test".to_string()),
                renew_ttl_secs: None,
                disable_cas: None,
                connection_timeout_ms: None,
                response_timeout_ms: None,
            },
        };

        let text_from_config = r#"
vault:
    server: "127.0.0.1:8200"
    token:
        from_config: "test"
        "#;

        let de_from_config: Config = serde_yaml::from_str(text_from_config).unwrap();
        assert_eq!(de_from_config, from_config);
        // Just assert that it can be serialized, not about to do string comparison
        serde_yaml::to_string(&from_config).unwrap();
    }

    #[test]
    fn test_vault_timeout_parsing() {
        let from_config = Config {
            vault: VaultConfig {
                namespace: None,
                server: "127.0.0.1:8200".to_string(),
                ca_certificate: None,
                token: Token::FromConfig("test".to_string()),
                renew_ttl_secs: None,
                disable_cas: None,
                connection_timeout_ms: Some(3000),
                response_timeout_ms: Some(5000),
            },
        };

        let text_from_config = r#"
vault:
    server: "127.0.0.1:8200"
    token:
        from_config: "test"
    connection_timeout_ms: 3000
    response_timeout_ms: 5000
        "#;

        let de_from_config: Config = serde_yaml::from_str(text_from_config).unwrap();
        assert_eq!(de_from_config, from_config);
        // Just assert that it can be serialized, no need to do string comparison
        serde_yaml::to_string(&from_config).unwrap();
    }

    #[test]
    fn test_token_disk_parsing() {
        let from_disk = Config {
            vault: VaultConfig {
                namespace: None,
                server: "127.0.0.1:8200".to_string(),
                ca_certificate: None,
                token: Token::FromDisk(PathBuf::from("/token")),
                renew_ttl_secs: None,
                disable_cas: None,
                connection_timeout_ms: None,
                response_timeout_ms: None,
            },
        };

        let text_from_disk = r#"
vault:
    server: "127.0.0.1:8200"
    token:
        from_disk: "/token"
        "#;

        let de_from_disk: Config = serde_yaml::from_str(text_from_disk).unwrap();
        assert_eq!(de_from_disk, from_disk);
        // Just assert that it can be serialized, not about to do string comparison
        serde_yaml::to_string(&from_disk).unwrap();
    }

    #[test]
    fn test_token_reading() {
        let temppath = diem_temppath::TempPath::new();
        temppath.create_as_file().unwrap();
        let mut file = File::create(temppath.path()).unwrap();
        file.write_all(b"disk_token").unwrap();

        let disk = Token::FromDisk(temppath.path().to_path_buf());
        assert_eq!("disk_token", disk.read_token().unwrap());

        let config = Token::FromConfig("config_token".to_string());
        assert_eq!("config_token", config.read_token().unwrap());
    }
}
