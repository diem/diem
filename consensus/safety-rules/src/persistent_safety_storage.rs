// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus_types::common::Round;
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_global_constants::{CONSENSUS_KEY, EPOCH, LAST_VOTED_ROUND, PREFERRED_ROUND, WAYPOINT};
use libra_secure_storage::{BoxedStorage, CryptoStorage, InMemoryStorage, KVStorage, Value};
use libra_types::waypoint::Waypoint;
use std::str::FromStr;

/// SafetyRules needs an abstract storage interface to act as a common utility for storing
/// persistent data to local disk, cloud, secrets managers, or even memory (for tests)
/// Any set function is expected to sync to the remote system before returning.
/// @TODO add access to private key from persistent store
/// @TODO add retrieval of private key based upon public key to persistent store
pub struct PersistentSafetyStorage {
    internal_store: BoxedStorage,
}

impl PersistentSafetyStorage {
    pub fn in_memory(private_key: Ed25519PrivateKey) -> Self {
        let storage = BoxedStorage::from(InMemoryStorage::new());
        Self::initialize(storage, private_key, Waypoint::default())
    }

    /// Use this to instantiate a PersistentStorage for a new data store, one that has no
    /// SafetyRules values set.
    pub fn initialize(
        mut internal_store: BoxedStorage,
        private_key: Ed25519PrivateKey,
        waypoint: Waypoint,
    ) -> Self {
        Self::initialize_(&mut internal_store, private_key, waypoint)
            .expect("Unable to initialize backend storage");
        Self { internal_store }
    }

    fn initialize_(
        internal_store: &mut BoxedStorage,
        private_key: Ed25519PrivateKey,
        waypoint: Waypoint,
    ) -> Result<()> {
        internal_store.import_private_key(CONSENSUS_KEY, private_key)?;
        internal_store.set(EPOCH, Value::U64(1))?;
        internal_store.set(LAST_VOTED_ROUND, Value::U64(0))?;
        internal_store.set(PREFERRED_ROUND, Value::U64(0))?;
        internal_store.set(WAYPOINT, Value::String(waypoint.to_string()))?;
        Ok(())
    }

    /// Use this to instantiate a PersistentStorage with an existing data store. This is intended
    /// for constructed environments.
    pub fn new(internal_store: BoxedStorage) -> Self {
        Self { internal_store }
    }

    pub fn consensus_key(&self) -> Result<Ed25519PrivateKey> {
        self.internal_store
            .export_private_key(CONSENSUS_KEY)
            .map_err(|e| e.into())
    }

    pub fn epoch(&self) -> Result<u64> {
        Ok(self.internal_store.get(EPOCH).and_then(|r| r.value.u64())?)
    }

    pub fn set_epoch(&mut self, epoch: u64) -> Result<()> {
        self.internal_store.set(EPOCH, Value::U64(epoch))?;
        Ok(())
    }

    pub fn last_voted_round(&self) -> Result<Round> {
        Ok(self
            .internal_store
            .get(LAST_VOTED_ROUND)
            .and_then(|r| r.value.u64())?)
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
            .and_then(|r| r.value.u64())?)
    }

    pub fn set_preferred_round(&mut self, preferred_round: Round) -> Result<()> {
        self.internal_store
            .set(PREFERRED_ROUND, Value::U64(preferred_round))?;
        Ok(())
    }

    pub fn waypoint(&self) -> Result<Waypoint> {
        let waypoint = self
            .internal_store
            .get(WAYPOINT)
            .and_then(|r| r.value.string())?;
        Waypoint::from_str(&waypoint)
            .map_err(|e| anyhow::anyhow!("Unable to parse waypoint: {}", e))
    }

    pub fn set_waypoint(&mut self, waypoint: &Waypoint) -> Result<()> {
        self.internal_store
            .set(WAYPOINT, Value::String(waypoint.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libra_types::validator_signer::ValidatorSigner;

    #[test]
    fn test() {
        let private_key = ValidatorSigner::from_int(0).private_key().clone();
        let mut storage = PersistentSafetyStorage::in_memory(private_key);
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
