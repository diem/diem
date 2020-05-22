// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf};

// JSON RPC endpoint related defaults
const DEFAULT_JSON_RPC_ENDPOINT: &str = "https://127.0.0.1:8080";

// Key manager timing related defaults
const DEFAULT_ROTATION_PERIOD_SECS: u64 = 604_800; // 1 week
const DEFAULT_SLEEP_PERIOD_SECS: u64 = 600; // 10 minutes
const DEFAULT_TXN_EXPIRATION_SECS: u64 = 3600; // 1 hour

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SecureConfig {
    pub key_manager: KeyManagerConfig,
}

impl SecureConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.key_manager.set_data_dir(data_dir);
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyManagerConfig {
    pub rotation_period_secs: u64,
    pub sleep_period_secs: u64,
    pub txn_expiration_secs: u64,

    pub json_rpc_endpoint: String,
    pub secure_backend: SecureBackend,
}

impl Default for KeyManagerConfig {
    fn default() -> KeyManagerConfig {
        KeyManagerConfig {
            rotation_period_secs: DEFAULT_ROTATION_PERIOD_SECS,
            sleep_period_secs: DEFAULT_SLEEP_PERIOD_SECS,
            txn_expiration_secs: DEFAULT_TXN_EXPIRATION_SECS,
            json_rpc_endpoint: DEFAULT_JSON_RPC_ENDPOINT.into(),
            secure_backend: SecureBackend::InMemoryStorage,
        }
    }
}

impl KeyManagerConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        if let SecureBackend::OnDiskStorage(backend) = &mut self.secure_backend {
            backend.set_data_dir(data_dir);
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SecureBackend {
    GitHub(GitHubConfig),
    InMemoryStorage,
    Vault(VaultConfig),
    OnDiskStorage(OnDiskStorageConfig),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GitHubConfig {
    /// The owner or account that hosts a repository
    pub owner: String,
    /// The repository where storage will mount
    pub repository: String,
    /// The authorization token for accessing the repository
    pub token: Token,
    /// A namespace is an optional portion of the path to a key stored within OnDiskStorage. For
    /// example, a key, S, without a namespace would be available in S, with a namespace, N, it
    /// would be in N/S.
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VaultConfig {
    /// Optional SSL Certificate for the vault host, this is expected to be a full path.
    pub ca_certificate: Option<PathBuf>,
    /// A namespace is an optional portion of the path to a key stored within Vault. For example,
    /// a secret, S, without a namespace would be available in secret/data/S, with a namespace, N, it
    /// would be in secret/data/N/S.
    pub namespace: Option<String>,
    /// Vault's URL, note: only HTTP is currently supported.
    pub server: String,
    /// The authorization token for accessing secrets
    pub token: Token,
}

impl VaultConfig {
    pub fn ca_certificate(&self) -> Result<String> {
        let path = self
            .ca_certificate
            .as_ref()
            .ok_or_else(|| anyhow!("No Certificate path"))?;
        read_file(path)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Token {
    FromConfig(TokenFromConfig),
    /// This is an absolute path and not relative to data_dir
    FromDisk(TokenFromDisk),
}

impl Token {
    pub fn new_config(token: String) -> Token {
        Token::FromConfig(TokenFromConfig { token })
    }

    pub fn new_disk(path: PathBuf) -> Token {
        Token::FromDisk(TokenFromDisk { path })
    }

    pub fn read_token(&self) -> Result<String> {
        match self {
            Token::FromDisk(from_disk) => read_file(&from_disk.path),
            Token::FromConfig(from_config) => Ok(from_config.token.clone()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TokenFromConfig {
    token: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TokenFromDisk {
    path: PathBuf,
}

impl Default for OnDiskStorageConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            path: PathBuf::from("secure_storage.toml"),
            data_dir: PathBuf::from("/opt/libra/data/common"),
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

fn read_file(path: &PathBuf) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
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
                token: Token::FromConfig(TokenFromConfig {
                    token: "test".to_string(),
                }),
            },
        };

        let text_from_config = "
[vault]
server = \"127.0.0.1:8200\"

[vault.token]
type = \"from_config\"
token = \"test\"
        ";

        let de_from_config: Config = toml::from_str(text_from_config).unwrap();
        assert_eq!(de_from_config, from_config);
        // Just assert that it can be serialized, not about to do string comparison
        toml::to_string(&from_config).unwrap();
    }

    #[test]
    fn test_token_disk_parsing() {
        let from_disk = Config {
            vault: VaultConfig {
                namespace: None,
                server: "127.0.0.1:8200".to_string(),
                ca_certificate: None,
                token: Token::FromDisk(TokenFromDisk {
                    path: PathBuf::from("/token"),
                }),
            },
        };

        let text_from_disk = "
[vault]
server = \"127.0.0.1:8200\"

[vault.token]
type = \"from_disk\"
path = \"/token\"
        ";

        let de_from_disk: Config = toml::from_str(text_from_disk).unwrap();
        assert_eq!(de_from_disk, from_disk);
        // Just assert that it can be serialized, not about to do string comparison
        toml::to_string(&from_disk).unwrap();
    }

    #[test]
    fn test_token_reading() {
        let temppath = libra_temppath::TempPath::new();
        temppath.create_as_file().unwrap();
        let mut file = File::create(temppath.path()).unwrap();
        file.write_all(b"disk_token").unwrap();

        let disk = Token::new_disk(temppath.path().to_path_buf());
        assert_eq!("disk_token", disk.read_token().unwrap());

        let config = Token::new_config("config_token".to_string());
        assert_eq!("config_token", config.read_token().unwrap());
    }
}
