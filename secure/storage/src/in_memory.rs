// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoKVStorage, Error, GetResponse, KVStorage};
use diem_time_service::{TimeService, TimeServiceTrait};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// InMemoryStorage represents a key value store that is purely in memory and intended for single
/// threads (or must be wrapped by a Arc<RwLock<>>). This provides no permission checks and simply
/// is a proof of concept to unblock building of applications without more complex data stores.
/// Internally, it retains all data, which means that it must make copies of all key material which
/// violates the Diem code base. It violates it because the anticipation is that data stores would
/// securely handle key material. This should not be used in production.
#[derive(Default)]
pub struct InMemoryStorage {
    data: HashMap<String, Vec<u8>>,
    time_service: TimeService,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::new_with_time_service(TimeService::real())
    }
}

impl InMemoryStorage {
    pub fn new_with_time_service(time_service: TimeService) -> Self {
        Self {
            data: HashMap::new(),
            time_service,
        }
    }
}

impl KVStorage for InMemoryStorage {
    fn available(&self) -> Result<(), Error> {
        Ok(())
    }

    fn get<V: DeserializeOwned>(&self, key: &str) -> Result<GetResponse<V>, Error> {
        let response = self
            .data
            .get(key)
            .ok_or_else(|| Error::KeyNotSet(key.to_string()))?;

        serde_json::from_slice(&response).map_err(|e| e.into())
    }

    fn set<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), Error> {
        let now = self.time_service.now().as_secs();
        self.data.insert(
            key.to_string(),
            serde_json::to_vec(&GetResponse::new(value, now))?,
        );
        Ok(())
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.data.clear();
        Ok(())
    }
}

impl CryptoKVStorage for InMemoryStorage {}
