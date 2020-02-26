// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Capability, Error, Identity, Policy, Storage, Value};
use libra_vault_client::{self as vault, Client};

/// VaultStorage utilizes Vault for maintaining encrypted, authenticated data for Libra. This
/// version currently matches the behavior of OnDiskStorage and InMemoryStorage. In the future,
/// Vault will be able to create keys, sign messages, and handle permissions across different
/// services. The specific vault service leveraged herein is called KV (Key Value) Secrets Engine -
/// Version 2 (https://www.vaultproject.io/api/secret/kv/kv-v2.html). So while Libra Secure Storage
/// calls pointers to data keys, Vault has actually a secret that contains multiple key value
/// pairs.
pub struct VaultStorage {
    pub client: Client,
}

impl VaultStorage {
    pub fn new(host: String, token: String) -> Self {
        Self {
            client: Client::new(host, token),
        }
    }

    /// Erase all secrets and keys from the vault storage. Use with caution.
    pub fn reset(&self) -> Result<(), Error> {
        let secrets = self.client.list_secrets("")?;
        for secret in secrets {
            self.client.delete_secret(&secret)?;
        }
        Ok(())
    }

    /// Retrieves a key from a given secret. Libra Secure Storage inserts each key into its own
    /// distinct secret store and thus the secret and key have the same identifier.
    fn get_secret(&self, key: &str) -> Result<Value, Error> {
        let value = self.client.read_secret(key, key)?;
        let v = Value::from_base64(&value).unwrap();
        Ok(v)
    }

    /// Inserts a key, value pair into a secret that shares the name of the key.
    fn set_secret(&self, key: &str, value: Value) -> Result<(), Error> {
        self.client.write_secret(key, key, &value.to_base64()?)?;
        Ok(())
    }

    /// Create a new policy in Vault, see the explanation for Policy for how the data is
    /// structured. Vault does not distingush a create and update. An update must first read the
    /// existing policy, amend the contents,  and then be applied via this API.
    fn set_policy(
        &self,
        policy_name: &str,
        key: &str,
        capabilities: &[Capability],
    ) -> Result<(), Error> {
        let path = format!("secret/data/{}", key);
        let mut vault_policy = self.client.read_policy(&policy_name).unwrap_or_default();
        let vault_capabilities: Vec<vault::Capability> = capabilities
            .iter()
            .map(|c| match c {
                Capability::Write => vault::Capability::Update,
                Capability::Read => vault::Capability::Read,
            })
            .collect();
        vault_policy.add_policy(&path, vault_capabilities);
        self.client.set_policy(policy_name, &vault_policy)?;
        Ok(())
    }
}

impl Storage for VaultStorage {
    fn available(&self) -> bool {
        self.client.unsealed().unwrap_or(false)
    }

    fn create(&mut self, key: &str, value: Value, policy: &Policy) -> Result<(), Error> {
        // Vault internally does not distinguish creation versus update except by permissions. So we
        // simulate that by first getting the key. If it doesn't exist, we're okay.
        match self.get_secret(&key) {
            Ok(_) => return Err(Error::KeyAlreadyExists(key.to_string())),
            Err(Error::KeyNotSet(_)) => (/* Expected this for new keys! */),
            Err(e) => return Err(e),
        }

        self.set_secret(&key, value)?;
        for permission in &policy.permissions {
            match &permission.id {
                Identity::User(id) => self.set_policy(id, key, &permission.capabilities)?,
                Identity::Anyone => self.set_policy("default", key, &permission.capabilities)?,
                Identity::NoOne => (),
            };
        }
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Value, Error> {
        self.get_secret(&key)
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        // Vault internally does not distinguish create versus udpate except by permissions. So we
        // simulate that by first getting the key. If it exists, we can update it.
        self.get_secret(&key)?;
        self.set_secret(&key, value)?;
        Ok(())
    }
}
