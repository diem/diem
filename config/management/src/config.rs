// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::StorageWrapper};
use libra_config::config;
use libra_types::chain_id::{self, ChainId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

/// A config file for working with management tooling.
///
/// # Example:
///
/// ```
/// use libra_management::config::ConfigPath;
/// use structopt::StructOpt;
///
/// #[derive(Clone, Debug, StructOpt)]
/// struct TestCommandLine {
///     #[structopt(long, required_unless("config"))]
///     test: Option<String>,
///     #[structopt(flatten)]
///     config: ConfigPath,
/// }
///
/// let config = "cmd --config test";
/// TestCommandLine::from_iter(config.split_whitespace());
///
/// let data  = "cmd --test test";
/// TestCommandLine::from_iter(data.split_whitespace());
///
/// // Unfortunately there's no easy way to catch these, so these are here purley for demo:
///
/// // let help = "cmd --help";
/// // let result = TestCommandLine::from_iter(help.split_whitespace());
///
/// // Output:
/// // ...
/// // OPTIONS:
/// //         --config <config>    Path to a libra-management configuration file
/// //         --test <test>
///
/// // let none  = "cmd";
/// // let result = TestCommandLine::from_iter(none.split_whitespace());
///
/// // Output:
/// // error: The following required arguments were not provided:
/// //     --test <test>
/// //
/// // USAGE:
/// //     cmd [OPTIONS] --test <test>
///```

/// Config for libra management tools
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "chain_id::deserialize_config_chain_id")]
    pub chain_id: ChainId,
    pub json_server: String,
    pub shared_backend: config::SecureBackend,
    pub validator_backend: config::SecureBackend,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_id: ChainId::default(),
            json_server: String::default(),
            shared_backend: config::SecureBackend::InMemoryStorage,
            validator_backend: config::SecureBackend::InMemoryStorage,
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Config, Error> {
        let reader = std::fs::File::open(path).map_err(|e| Error::ConfigError(e.to_string()))?;
        serde_yaml::from_reader(reader).map_err(|e| Error::ConfigError(e.to_string()))
    }

    pub fn override_chain_id(mut self, chain_id: Option<ChainId>) -> Self {
        if let Some(chain_id) = chain_id {
            self.chain_id = chain_id;
        }
        self
    }

    pub fn override_json_server(mut self, json_server: &Option<String>) -> Self {
        if let Some(json_server) = json_server {
            self.json_server = json_server.clone();
        }
        self
    }

    pub fn override_shared_backend(
        mut self,
        shared_backend: &Option<crate::secure_backend::SecureBackend>,
    ) -> Result<Self, Error> {
        if let Some(backend) = &shared_backend {
            self.shared_backend = std::convert::TryInto::try_into(backend.clone())?;
        }
        Ok(self)
    }

    pub fn override_validator_backend(
        mut self,
        validator_backend: &Option<crate::secure_backend::SecureBackend>,
    ) -> Result<Self, Error> {
        if let Some(backend) = &validator_backend {
            self.validator_backend = std::convert::TryInto::try_into(backend.clone())?;
        }
        Ok(self)
    }

    pub fn shared_backend(&self) -> StorageWrapper {
        StorageWrapper {
            storage_name: "shared",
            storage: std::convert::From::from(&self.shared_backend),
        }
    }

    pub fn shared_backend_with_namespace(&self, namespace: String) -> StorageWrapper {
        let mut shared_backend = self.shared_backend.clone();
        match &mut shared_backend {
            config::SecureBackend::GitHub(config) => config.namespace = Some(namespace),
            config::SecureBackend::InMemoryStorage => panic!("Unsupported namespace for InMemory"),
            config::SecureBackend::Vault(config) => config.namespace = Some(namespace),
            config::SecureBackend::OnDiskStorage(config) => config.namespace = Some(namespace),
        };
        StorageWrapper {
            storage_name: "shared",
            storage: std::convert::From::from(&shared_backend),
        }
    }

    pub fn validator_backend(&self) -> StorageWrapper {
        StorageWrapper {
            storage_name: "validator",
            storage: std::convert::From::from(&self.validator_backend),
        }
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct ConfigPath {
    /// Path to a libra-management configuration file
    #[structopt(long)]
    config: Option<PathBuf>,
}

impl ConfigPath {
    pub fn load(&self) -> Result<Config, Error> {
        if let Some(path) = &self.config {
            Config::load(path)
        } else {
            Ok(Config::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libra_config::config::{SecureBackend, Token, VaultConfig};
    use libra_types::chain_id::NamedChain;

    #[test]
    fn example() {
        let config_value = Config {
            chain_id: ChainId::new(NamedChain::MAINNET.id()),
            json_server: "http://127.0.0.1:8080".to_string(),
            shared_backend: SecureBackend::Vault(VaultConfig {
                namespace: None,
                server: "127.0.0.1:8200".to_string(),
                ca_certificate: None,
                token: Token::FromConfig("test".to_string()),
            }),
            validator_backend: SecureBackend::Vault(VaultConfig {
                namespace: None,
                server: "127.0.0.1:8200".to_string(),
                ca_certificate: None,
                token: Token::FromConfig("test".to_string()),
            }),
        };

        let config_text = r#"
chain_id: "MAINNET"
json_server: "http://127.0.0.1:8080"
shared_backend:
    type: "vault"
    server: "127.0.0.1:8200"
    token:
        from_config: "test"
validator_backend:
    type: "vault"
    server: "127.0.0.1:8200"
    token:
        from_config: "test"
        "#;

        let de_config: Config = serde_yaml::from_str(config_text).unwrap();
        assert_eq!(de_config, config_value);
        // Just assert that it can be serialized, not about to do string comparison
        serde_yaml::to_string(&config_value).unwrap();
    }
}
