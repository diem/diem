// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Storage, Value};
use libra_vault_client::Client;

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
}

impl Storage for VaultStorage {
    fn available(&self) -> bool {
        self.client.unsealed().unwrap_or(false)
    }

    fn create(&mut self, key: &str, value: Value, _policy: &Policy) -> Result<(), Error> {
        // Vault internally does not distinguish creation versus update except by permissions. So we
        // simulate that by first getting the key. If it doesn't exist, we're okay and return back a
        // fake, but ignored value.
        self.get_secret(&key).or_else(|e| {
            if let Error::KeyNotSet(_) = e {
                Ok(Value::U64(0))
            } else {
                Err(e)
            }
        })?;

        self.set_secret(&key, value)?;
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
