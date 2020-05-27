// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::{KeyManagerConfig as KMConfig, SecureBackend, Token, VaultConfig};
use libra_types::account_address::AccountAddress;

pub struct KeyManagerConfig {
    rotation_period_secs: Option<u64>,
    sleep_period_secs: Option<u64>,
    txn_expiration_secs: Option<u64>,

    json_rpc_endpoint: String,
    validator_account: AccountAddress,

    vault_host: String,
    vault_namespace: Option<String>,
    vault_token: String,

    template: KMConfig,
}

impl Default for KeyManagerConfig {
    fn default() -> Self {
        let template = KMConfig::default();
        Self {
            rotation_period_secs: None,
            sleep_period_secs: None,
            txn_expiration_secs: None,
            json_rpc_endpoint: template.json_rpc_endpoint.clone(),
            validator_account: template.validator_account,
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

    pub fn json_rpc_endpoint(&mut self, json_rpc_endpoint: String) -> &mut Self {
        self.json_rpc_endpoint = json_rpc_endpoint;
        self
    }

    pub fn time_constants(
        &mut self,
        rotation_period_secs: Option<u64>,
        sleep_period_secs: Option<u64>,
        txn_expiration_secs: Option<u64>,
    ) -> &mut Self {
        self.rotation_period_secs = rotation_period_secs;
        self.sleep_period_secs = sleep_period_secs;
        self.txn_expiration_secs = txn_expiration_secs;
        self
    }

    pub fn validator_account(&mut self, validator_account: AccountAddress) -> &mut Self {
        self.validator_account = validator_account;
        self
    }

    pub fn vault_storage(
        &mut self,
        vault_host: String,
        vault_namespace: Option<String>,
        vault_token: String,
    ) -> &mut Self {
        self.vault_host = vault_host;
        self.vault_namespace = vault_namespace;
        self.vault_token = vault_token;
        self
    }

    pub fn template(&mut self, template: KMConfig) -> &mut Self {
        self.template = template;
        self
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
        key_manager_config.validator_account = self.validator_account;

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
        let validator_account = AccountAddress::default();

        let mut key_manager_config = KeyManagerConfig::new();
        let key_manager_config = key_manager_config
            .json_rpc_endpoint(json_rpc_endpoint.into())
            .time_constants(Some(rotation_period_secs), None, None)
            .validator_account(validator_account)
            .vault_storage(vault_host.into(), None, vault_token.into())
            .build()
            .unwrap();

        assert_eq!(json_rpc_endpoint, key_manager_config.json_rpc_endpoint);
        assert_eq!(validator_account, key_manager_config.validator_account);
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
