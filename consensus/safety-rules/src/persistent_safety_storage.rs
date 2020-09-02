// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    logging::{self, LogEntry, LogEvent},
    Error,
};
use consensus_types::{common::Author, safety_data::SafetyData};
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use libra_global_constants::{CONSENSUS_KEY, EXECUTION_KEY, OWNER_ACCOUNT, SAFETY_DATA, WAYPOINT};
use libra_logger::prelude::*;
use libra_secure_storage::{CryptoStorage, InMemoryStorage, KVStorage, Storage};
use libra_types::waypoint::Waypoint;

/// SafetyRules needs an abstract storage interface to act as a common utility for storing
/// persistent data to local disk, cloud, secrets managers, or even memory (for tests)
/// Any set function is expected to sync to the remote system before returning.
/// @TODO add access to private key from persistent store
/// @TODO add retrieval of private key based upon public key to persistent store
pub struct PersistentSafetyStorage {
    internal_store: Storage,
}

impl PersistentSafetyStorage {
    pub fn in_memory(
        consensus_private_key: Ed25519PrivateKey,
        execution_private_key: Ed25519PrivateKey,
    ) -> Self {
        let storage = Storage::from(InMemoryStorage::new());
        Self::initialize(
            storage,
            Author::random(),
            consensus_private_key,
            execution_private_key,
            Waypoint::default(),
        )
    }

    /// Use this to instantiate a PersistentStorage for a new data store, one that has no
    /// SafetyRules values set.
    pub fn initialize(
        mut internal_store: Storage,
        author: Author,
        consensus_private_key: Ed25519PrivateKey,
        execution_private_key: Ed25519PrivateKey,
        waypoint: Waypoint,
    ) -> Self {
        Self::initialize_(
            &mut internal_store,
            author,
            consensus_private_key,
            execution_private_key,
            waypoint,
        )
        .expect("Unable to initialize backend storage");
        Self { internal_store }
    }

    fn initialize_(
        internal_store: &mut Storage,
        author: Author,
        consensus_private_key: Ed25519PrivateKey,
        execution_private_key: Ed25519PrivateKey,
        waypoint: Waypoint,
    ) -> Result<(), Error> {
        let result = internal_store.import_private_key(CONSENSUS_KEY, consensus_private_key);
        // Attempting to re-initialize existing storage. This can happen in environments like
        // cluster test. Rather than be rigid here, leave it up to the developer to detect
        // inconsistencies or why they did not reset storage between rounds. Do not repeat the
        // checks again below, because it is just too strange to have a partially configured
        // storage.
        if let Err(libra_secure_storage::Error::KeyAlreadyExists(_)) = result {
            warn!("Attempted to re-initialize existing storage");
            return Ok(());
        }

        internal_store.import_private_key(EXECUTION_KEY, execution_private_key)?;
        internal_store.set(SAFETY_DATA, SafetyData::new(1, 0, 0, None))?;
        internal_store.set(OWNER_ACCOUNT, author)?;
        internal_store.set(WAYPOINT, waypoint)?;
        Ok(())
    }

    /// Use this to instantiate a PersistentStorage with an existing data store. This is intended
    /// for constructed environments.
    pub fn new(internal_store: Storage) -> Self {
        Self { internal_store }
    }

    pub fn author(&self) -> Result<Author, Error> {
        Ok(self.internal_store.get(OWNER_ACCOUNT).map(|v| v.value)?)
    }

    pub fn consensus_key_for_version(
        &self,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        Ok(self
            .internal_store
            .export_private_key_for_version(CONSENSUS_KEY, version)?)
    }

    pub fn execution_public_key(&self) -> Result<Ed25519PublicKey, Error> {
        Ok(self
            .internal_store
            .get_public_key(EXECUTION_KEY)
            .map(|r| r.public_key)?)
    }

    pub fn safety_data(&self) -> Result<SafetyData, Error> {
        Ok(self.internal_store.get(SAFETY_DATA).map(|v| v.value)?)
    }

    pub fn set_safety_data(&mut self, data: SafetyData) -> Result<(), Error> {
        counters::set_state("epoch", data.epoch as i64);
        counters::set_state("last_voted_round", data.last_voted_round as i64);
        counters::set_state("preferred_round", data.preferred_round as i64);
        self.internal_store.set(SAFETY_DATA, data)?;
        Ok(())
    }

    pub fn waypoint(&self) -> Result<Waypoint, Error> {
        Ok(self.internal_store.get(WAYPOINT).map(|v| v.value)?)
    }

    pub fn set_waypoint(&mut self, waypoint: &Waypoint) -> Result<(), Error> {
        self.internal_store.set(WAYPOINT, waypoint)?;
        info!(
            logging::SafetyLogSchema::new(LogEntry::Waypoint, LogEvent::Update)
                .waypoint(*waypoint)
                .into_struct_log()
        );
        Ok(())
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn internal_store(&mut self) -> &mut Storage {
        &mut self.internal_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libra_crypto::Uniform;
    use libra_types::validator_signer::ValidatorSigner;

    #[test]
    fn test() {
        let private_key = ValidatorSigner::from_int(0).private_key().clone();
        let mut storage = PersistentSafetyStorage::in_memory(
            private_key,
            Ed25519PrivateKey::generate_for_testing(),
        );
        let safety_data = storage.safety_data().unwrap();
        assert_eq!(safety_data.epoch, 1);
        assert_eq!(safety_data.last_voted_round, 0);
        assert_eq!(safety_data.preferred_round, 0);
        storage
            .set_safety_data(SafetyData::new(9, 8, 1, None))
            .unwrap();
        let safety_data = storage.safety_data().unwrap();
        assert_eq!(safety_data.epoch, 9);
        assert_eq!(safety_data.last_voted_round, 8);
        assert_eq!(safety_data.preferred_round, 1);
    }
}
