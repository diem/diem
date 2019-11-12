// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::common::Round;
use libra_config::config::PersistableConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[cfg(test)]
use tempfile::NamedTempFile;

/// SafetyRules needs an abstract storage interface to act as a common utility for storing
/// persistent data to local disk, cloud, secrets managers, or even memory (for tests)
/// Any set function is expected to sync to the remote system before returning.
/// @TODO add access to private key from persistent store
/// @TODO add retrieval of private key based upon public key to persistent store
pub trait PersistentStorage: Send + Sync {
    fn epoch(&self) -> u64;
    fn set_epoch(&mut self, epoch: u64);
    fn last_voted_round(&self) -> Round;
    fn set_last_voted_round(&mut self, last_voted_round: Round);
    fn preferred_round(&self) -> Round;
    fn set_preferred_round(&mut self, last_voted_round: Round);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InMemoryStorage {
    epoch: u64,
    last_voted_round: Round,
    preferred_round: Round,
}

impl InMemoryStorage {
    pub fn new(epoch: u64, last_voted_round: Round, preferred_round: Round) -> Self {
        Self {
            epoch,
            last_voted_round,
            preferred_round,
        }
    }

    pub fn default() -> Self {
        Self {
            epoch: 1,
            last_voted_round: 0,
            preferred_round: 0,
        }
    }

    pub fn default_storage() -> Box<dyn PersistentStorage> {
        Box::new(Self::default())
    }
}

impl PersistentStorage for InMemoryStorage {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    fn preferred_round(&self) -> Round {
        self.preferred_round
    }

    fn set_preferred_round(&mut self, preferred_round: Round) {
        self.preferred_round = preferred_round;
    }

    fn last_voted_round(&self) -> Round {
        self.last_voted_round
    }

    fn set_last_voted_round(&mut self, last_voted_round: Round) {
        self.last_voted_round = last_voted_round;
    }
}

#[test]
fn test_in_memory_storage() {
    let mut storage: Box<dyn PersistentStorage> = InMemoryStorage::default_storage();
    assert_eq!(storage.epoch(), 1);
    assert_eq!(storage.last_voted_round(), 0);
    assert_eq!(storage.preferred_round(), 0);
    storage.set_epoch(9);
    storage.set_last_voted_round(8);
    storage.set_preferred_round(1);
    assert_eq!(storage.epoch(), 9);
    assert_eq!(storage.last_voted_round(), 8);
    assert_eq!(storage.preferred_round(), 1);
}

pub struct OnDiskStorage {
    file_path: PathBuf,
    internal_data: InMemoryStorage,
}

impl OnDiskStorage {
    pub fn new_storage(file_path: PathBuf) -> Box<dyn PersistentStorage> {
        let internal_data = InMemoryStorage::load_config(file_path.clone());
        Box::new(Self {
            file_path,
            internal_data,
        })
    }

    pub fn default_storage(file_path: PathBuf) -> Box<dyn PersistentStorage> {
        if file_path.exists() {
            return Self::new_storage(file_path);
        }

        let internal_data = InMemoryStorage::default();
        internal_data.save_config(file_path.clone());
        Box::new(Self {
            file_path,
            internal_data,
        })
    }
}

impl PersistentStorage for OnDiskStorage {
    fn epoch(&self) -> u64 {
        self.internal_data.epoch()
    }

    fn set_epoch(&mut self, epoch: u64) {
        self.internal_data.set_epoch(epoch);
        self.internal_data.save_config(self.file_path.clone());
    }

    fn preferred_round(&self) -> Round {
        self.internal_data.preferred_round()
    }

    fn set_preferred_round(&mut self, preferred_round: Round) {
        self.internal_data.set_preferred_round(preferred_round);
        self.internal_data.save_config(self.file_path.clone());
    }

    fn last_voted_round(&self) -> Round {
        self.internal_data.last_voted_round()
    }

    fn set_last_voted_round(&mut self, last_voted_round: Round) {
        self.internal_data.set_last_voted_round(last_voted_round);
        self.internal_data.save_config(self.file_path.clone());
    }
}

#[test]
fn test_on_disk_storage() {
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let mut storage: Box<dyn PersistentStorage> = OnDiskStorage::default_storage(file_path.clone());
    assert_eq!(storage.epoch(), 1);
    assert_eq!(storage.last_voted_round(), 0);
    assert_eq!(storage.preferred_round(), 0);
    storage.set_epoch(9);
    storage.set_last_voted_round(8);
    storage.set_preferred_round(1);
    assert_eq!(storage.epoch(), 9);
    assert_eq!(storage.last_voted_round(), 8);
    assert_eq!(storage.preferred_round(), 1);

    let storage: Box<dyn PersistentStorage> = OnDiskStorage::default_storage(file_path);
    assert_eq!(storage.epoch(), 9);
    assert_eq!(storage.last_voted_round(), 8);
    assert_eq!(storage.preferred_round(), 1);
}
