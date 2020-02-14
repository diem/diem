// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Capability, Error, Identity, Policy, Storage, Value};
use libra_secure_storage_aws::SecretsManagerStorage;

pub struct AwsStorage {
    pub client: SecretsManagerStorage,
}

impl AwsStorage {
    pub fn new() -> Self {
        Self {
            client: SecretsManagerStorage::new("us-west-2"),
        }
    }

    /// Erase all secrets and keys from the vault storage. Use with caution.
    pub fn reset(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Retrieves a key from a given secret. Libra Secure Storage inserts each key into its own
    /// distinct secret store and thus the secret and key have the same identifier.
    fn get_secret(&self, key: &str) -> Result<Value, Error> {
        Ok(v)
    }

    /// Inserts a key, value pair into a secret that shares the name of the key.
    fn set_secret(&self, key: &str, value: Value) -> Result<(), Error> {
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
        Ok(())
    }
}

impl Storage for AwsStorage {
    fn available(&self) -> bool {
        self.client.unsealed().unwrap_or(false)
    }

    fn create(&mut self, key: &str, value: Value, policy: &Policy) -> Result<(), Error> {
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
