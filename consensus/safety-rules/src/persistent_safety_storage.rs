// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    logging::{self, LogEntry, LogEvent},
    Error,
};
use consensus_types::{common::Author, safety_data::SafetyData};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
};
use diem_global_constants::{CONSENSUS_KEY, EXECUTION_KEY, OWNER_ACCOUNT, SAFETY_DATA, WAYPOINT};
use diem_logger::prelude::*;
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_types::waypoint::Waypoint;
use serde::Serialize;

/// SafetyRules needs an abstract storage interface to act as a common utility for storing
/// persistent data to local disk, cloud, secrets managers, or even memory (for tests)
/// Any set function is expected to sync to the remote system before returning.
///
/// Note: cached_safety_data is a local in-memory copy of SafetyData. As SafetyData should
/// only ever be used by safety rules, we maintain an in-memory copy to avoid issuing reads
/// to the internal storage if the SafetyData hasn't changed. On writes, we update the
/// cache and internal storage.
pub struct PersistentSafetyStorage {
    enable_cached_safety_data: bool,
    cached_safety_data: Option<SafetyData>,
    internal_store: Storage,
}

impl PersistentSafetyStorage {
    /// Use this to instantiate a PersistentStorage for a new data store, one that has no
    /// SafetyRules values set.
    pub fn initialize(
        mut internal_store: Storage,
        author: Author,
        consensus_private_key: Ed25519PrivateKey,
        execution_private_key: Ed25519PrivateKey,
        waypoint: Waypoint,
        enable_cached_safety_data: bool,
    ) -> Self {
        let safety_data = SafetyData::new(1, 0, 0, None);
        Self::initialize_(
            &mut internal_store,
            safety_data.clone(),
            author,
            consensus_private_key,
            execution_private_key,
            waypoint,
        )
        .expect("Unable to initialize backend storage");
        Self {
            enable_cached_safety_data,
            cached_safety_data: Some(safety_data),
            internal_store,
        }
    }

    fn initialize_(
        internal_store: &mut Storage,
        safety_data: SafetyData,
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
        if let Err(diem_secure_storage::Error::KeyAlreadyExists(_)) = result {
            warn!("Attempted to re-initialize existing storage");
            return Ok(());
        }

        internal_store.import_private_key(EXECUTION_KEY, execution_private_key)?;
        internal_store.set(SAFETY_DATA, safety_data)?;
        internal_store.set(OWNER_ACCOUNT, author)?;
        internal_store.set(WAYPOINT, waypoint)?;
        Ok(())
    }

    /// Use this to instantiate a PersistentStorage with an existing data store. This is intended
    /// for constructed environments.
    pub fn new(internal_store: Storage, enable_cached_safety_data: bool) -> Self {
        Self {
            enable_cached_safety_data,
            cached_safety_data: None,
            internal_store,
        }
    }

    pub fn author(&self) -> Result<Author, Error> {
        let _timer = counters::start_timer("get", OWNER_ACCOUNT);
        Ok(self.internal_store.get(OWNER_ACCOUNT).map(|v| v.value)?)
    }

    pub fn consensus_key_for_version(
        &self,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        let _timer = counters::start_timer("get", CONSENSUS_KEY);
        Ok(self
            .internal_store
            .export_private_key_for_version(CONSENSUS_KEY, version)?)
    }

    pub fn execution_public_key(&self) -> Result<Ed25519PublicKey, Error> {
        let _timer = counters::start_timer("get", EXECUTION_KEY);
        Ok(self
            .internal_store
            .get_public_key(EXECUTION_KEY)
            .map(|r| r.public_key)?)
    }

    pub fn sign<T: Serialize + CryptoHash>(
        &self,
        key_name: String,
        key_version: Ed25519PublicKey,
        message: &T,
    ) -> Result<Ed25519Signature, Error> {
        Ok(self
            .internal_store
            .sign_using_version(&key_name, key_version, message)?)
    }

    pub fn safety_data(&mut self) -> Result<SafetyData, Error> {
        if !self.enable_cached_safety_data {
            let _timer = counters::start_timer("get", SAFETY_DATA);
            return self.internal_store.get(SAFETY_DATA).map(|v| v.value)?;
        }

        if let Some(cached_safety_data) = self.cached_safety_data.clone() {
            Ok(cached_safety_data)
        } else {
            let _timer = counters::start_timer("get", SAFETY_DATA);
            let safety_data: SafetyData = self.internal_store.get(SAFETY_DATA).map(|v| v.value)?;
            self.cached_safety_data = Some(safety_data.clone());
            Ok(safety_data)
        }
    }

    pub fn set_safety_data(&mut self, data: SafetyData) -> Result<(), Error> {
        let _timer = counters::start_timer("set", SAFETY_DATA);
        counters::set_state("epoch", data.epoch as i64);
        counters::set_state("last_voted_round", data.last_voted_round as i64);
        counters::set_state("preferred_round", data.preferred_round as i64);

        match self.internal_store.set(SAFETY_DATA, data.clone()) {
            Ok(_) => {
                self.cached_safety_data = Some(data);
                Ok(())
            }
            Err(error) => {
                self.cached_safety_data = None;
                Err(Error::SecureStorageUnexpectedError(error.to_string()))
            }
        }
    }

    pub fn waypoint(&self) -> Result<Waypoint, Error> {
        let _timer = counters::start_timer("get", WAYPOINT);
        Ok(self.internal_store.get(WAYPOINT).map(|v| v.value)?)
    }

    pub fn set_waypoint(&mut self, waypoint: &Waypoint) -> Result<(), Error> {
        let _timer = counters::start_timer("set", WAYPOINT);
        self.internal_store.set(WAYPOINT, waypoint)?;
        info!(
            logging::SafetyLogSchema::new(LogEntry::Waypoint, LogEvent::Update).waypoint(*waypoint)
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
    use diem_crypto::Uniform;
    use diem_secure_storage::InMemoryStorage;
    use diem_types::validator_signer::ValidatorSigner;

    #[test]
    fn test() {
        let consensus_private_key = ValidatorSigner::from_int(0).private_key().clone();
        let storage = Storage::from(InMemoryStorage::new());
        let mut safety_storage = PersistentSafetyStorage::initialize(
            storage,
            Author::random(),
            consensus_private_key,
            Ed25519PrivateKey::generate_for_testing(),
            Waypoint::default(),
            true,
        );

        let safety_data = safety_storage.safety_data().unwrap();
        assert_eq!(safety_data.epoch, 1);
        assert_eq!(safety_data.last_voted_round, 0);
        assert_eq!(safety_data.preferred_round, 0);

        safety_storage
            .set_safety_data(SafetyData::new(9, 8, 1, None))
            .unwrap();

        let safety_data = safety_storage.safety_data().unwrap();
        assert_eq!(safety_data.epoch, 9);
        assert_eq!(safety_data.last_voted_round, 8);
        assert_eq!(safety_data.preferred_round, 1);
    }
}
