// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::{KeyManagerConfig as KMConfig, SecureBackend, Token, VaultConfig};

pub struct KeyManagerConfig {
    pub rotation_period_secs: Option<u64>,
    pub sleep_period_secs: Option<u64>,
    pub txn_expiration_secs: Option<u64>,

    pub json_rpc_endpoint: String,

    pub vault_host: String,
    pub vault_namespace: Option<String>,
    pub vault_token: String,

    pub template: KMConfig,
}

impl Default for KeyManagerConfig {
    fn default() -> Self {
        let template = KMConfig::default();
        Self {
            rotation_period_secs: None,
            sleep_period_secs: None,
            txn_expiration_secs: None,
            json_rpc_endpoint: template.json_rpc_endpoint.clone(),
            vault_host: "127.0.0.1:8200".to_string(),
            vault_namespace: None,
            vault_token: "root_token".to_string(),
            template,
        }
    }
}

impl KeyManagerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> Result<KMConfig> {
        let mut key_manager_config = self.template.clone();
        key_manager_config.json_rpc_endpoint = self.json_rpc_endpoint.clone();
        key_manager_config.secure_backend = SecureBackend::Vault(VaultConfig {
            ca_certificate: None,
            namespace: self.vault_namespace.clone(),
            server: self.vault_host.clone(),
            token: Token::new_config(self.vault_token.clone()),
        });

        if let Some(rotation_period_secs) = &self.rotation_period_secs {
            key_manager_config.rotation_period_secs = *rotation_period_secs;
        }
        if let Some(sleep_period_secs) = &self.sleep_period_secs {
            key_manager_config.sleep_period_secs = *sleep_period_secs;
        }
        if let Some(txn_expiration_secs) = &self.txn_expiration_secs {
            key_manager_config.txn_expiration_secs = *txn_expiration_secs;
        }

        Ok(key_manager_config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_generation() {
        let json_rpc_endpoint = "http://127.12.12.12:7873";
        let rotation_period_secs = 100;
        let vault_host = "182.0.0.1:8080";
        let vault_token = "root_token";

        let mut key_manager_config = KeyManagerConfig::new();
        key_manager_config.json_rpc_endpoint = json_rpc_endpoint.into();
        key_manager_config.rotation_period_secs = Some(rotation_period_secs);
        key_manager_config.vault_host = vault_host.into();
        key_manager_config.vault_token = vault_token.into();

        let key_manager_config = key_manager_config.build().unwrap();

        assert_eq!(json_rpc_endpoint, key_manager_config.json_rpc_endpoint);
        assert_eq!(
            rotation_period_secs,
            key_manager_config.rotation_period_secs
        );
        if let SecureBackend::Vault(vault_config) = key_manager_config.secure_backend {
            assert_eq!(vault_host, vault_config.server);
            assert!(vault_config.namespace.is_none());
            assert_eq!(vault_token, vault_config.token.read_token().unwrap());
        } else {
            panic!("Invalid secure storage backend found for key manager!");
        }
    }
}
