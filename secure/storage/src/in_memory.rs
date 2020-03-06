// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, policy::Policy, storage::Storage, value::Value};
use std::collections::HashMap;

/// InMemoryStorage represents a key value store that is purely in memory and intended for single
/// threads (or must be wrapped by a Arc<RwLock<>>). This provides no permission checks and simply
/// is a proof of concept to unblock building of applications without more complex data stores.
/// Internally, it retains all data, which means that it must make copies of all key material which
/// violates the Libra code base. It violates it because the anticipation is that data stores would
/// securely handle key material. This should not be used in production.
#[derive(Default)]
pub struct InMemoryStorage {
    data: HashMap<String, Value>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for InMemoryStorage {
    fn available(&self) -> bool {
        true
    }

    fn create(&mut self, key: &str, value: Value, _policy: &Policy) -> Result<(), Error> {
        if self.data.contains_key(key) {
            return Err(Error::KeyAlreadyExists(key.to_string()));
        }
        self.data.insert(key.to_string(), value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Value, Error> {
        let value = self
            .data
            .get(key)
            .ok_or_else(|| Error::KeyNotSet(key.to_string()))?;

        let value = match value {
            Value::Ed25519PrivateKey(value) => {
                // Hack because Ed25519PrivateKey does not support clone / copy
                let bytes = lcs::to_bytes(value)?;
                let key = lcs::from_bytes(&bytes)?;
                Value::Ed25519PrivateKey(key)
            }
            Value::HashValue(value) => Value::HashValue(*value),
            Value::U64(value) => Value::U64(*value),
        };

        Ok(value)
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        if !self.data.contains_key(key) {
            return Err(Error::KeyNotSet(key.to_string()));
        }
        self.data.insert(key.to_string(), value);
        Ok(())
    }

    #[cfg(test)]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.data.clear();
        Ok(())
    }
}
