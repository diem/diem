// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus_types::common::Round;
use libra_secure_storage::{InMemoryStorage, Permissions, Storage, Value};

/// SafetyRules needs an abstract storage interface to act as a common utility for storing
/// persistent data to local disk, cloud, secrets managers, or even memory (for tests)
/// Any set function is expected to sync to the remote system before returning.
/// @TODO add access to private key from persistent store
/// @TODO add retrieval of private key based upon public key to persistent store
pub struct PersistentStorage {
    internal_store: Box<dyn Storage>,
}

const EPOCH: &str = "epoch";
const LAST_VOTED_ROUND: &str = "last_voted_round";
const PREFERRED_ROUND: &str = "preferred_round";

impl PersistentStorage {
    pub fn in_memory() -> Self {
        let storage = Box::new(InMemoryStorage::new());
        Self::initialize(storage)
    }

    /// Use this to instantiate a PersistentStorage for a new data store, one that has no
    /// SafetyRules values set.
    pub fn initialize(mut internal_store: Box<dyn Storage>) -> Self {
        let perms = Permissions::anyone();
        internal_store
            .create_if_not_exists(EPOCH, Value::U64(1), &perms)
            .expect("Unable to initialize backend storage");
        internal_store
            .create_if_not_exists(LAST_VOTED_ROUND, Value::U64(0), &perms)
            .expect("Unable to initialize backend storage");
        internal_store
            .create_if_not_exists(PREFERRED_ROUND, Value::U64(0), &perms)
            .expect("Unable to initialize backend storage");
        Self { internal_store }
    }

    /// Use this to instantiate a PersistentStorage with an existing data store. This is intended
    /// for constructed environments.
    pub fn new(internal_store: Box<dyn Storage>) -> Self {
        Self { internal_store }
    }

    pub fn epoch(&self) -> Result<u64> {
        Ok(self
            .internal_store
            .get(EPOCH)
            .and_then(|value| value.u64())?)
    }

    pub fn set_epoch(&mut self, epoch: u64) -> Result<()> {
        self.internal_store.set(EPOCH, Value::U64(epoch))?;
        Ok(())
    }

    pub fn last_voted_round(&self) -> Result<Round> {
        Ok(self
            .internal_store
            .get(LAST_VOTED_ROUND)
            .and_then(|value| value.u64())?)
    }

    pub fn set_last_voted_round(&mut self, last_voted_round: Round) -> Result<()> {
        self.internal_store
            .set(LAST_VOTED_ROUND, Value::U64(last_voted_round))?;
        Ok(())
    }

    pub fn preferred_round(&self) -> Result<Round> {
        Ok(self
            .internal_store
            .get(PREFERRED_ROUND)
            .and_then(|value| value.u64())?)
    }

    pub fn set_preferred_round(&mut self, preferred_round: Round) -> Result<()> {
        self.internal_store
            .set(PREFERRED_ROUND, Value::U64(preferred_round))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libra_secure_storage::InMemoryStorage;

    #[test]
    fn test() {
        let internal = Box::new(InMemoryStorage::new());
        let mut storage = PersistentStorage::initialize(internal);
        assert_eq!(storage.epoch().unwrap(), 1);
        assert_eq!(storage.last_voted_round().unwrap(), 0);
        assert_eq!(storage.preferred_round().unwrap(), 0);
        storage.set_epoch(9).unwrap();
        storage.set_last_voted_round(8).unwrap();
        storage.set_preferred_round(1).unwrap();
        assert_eq!(storage.epoch().unwrap(), 9);
        assert_eq!(storage.last_voted_round().unwrap(), 8);
        assert_eq!(storage.preferred_round().unwrap(), 1);
    }
}
