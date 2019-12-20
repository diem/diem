// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus_types::common::Round;
use libra_config::config::PersistableConfig;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, path::PathBuf};
#[cfg(test)]
use tempfile::NamedTempFile;

/// SafetyRules needs an abstract storage interface to act as a common utility for storing
/// persistent data to local disk, cloud, secrets managers, or even memory (for tests)
/// Any set function is expected to sync to the remote system before returning.
/// @TODO add access to private key from persistent store
/// @TODO add retrieval of private key based upon public key to persistent store
pub trait PersistentStorage: Send + Sync {
    fn epoch(&self) -> u64;
    fn set_epoch(&mut self, epoch: u64) -> Result<()>;
    fn last_voted_round(&self) -> Round;
    fn set_last_voted_round(&mut self, last_voted_round: Round) -> Result<()>;
    fn preferred_round(&self) -> Round;
    fn set_preferred_round(&mut self, last_voted_round: Round) -> Result<()>;
}

#[derive(Debug, Deserialize, Eq, PartialEq, PartialOrd, Serialize)]
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

    pub fn default_storage() -> Box<dyn PersistentStorage> {
        Box::new(Self::default())
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self {
            epoch: 1,
            last_voted_round: 0,
            preferred_round: 0,
        }
    }
}

impl Ord for InMemoryStorage {
    fn cmp(&self, other: &InMemoryStorage) -> Ordering {
        let epoch = self.epoch.cmp(&other.epoch);
        if epoch != Ordering::Equal {
            return epoch;
        }

        let last_voted_round = self.last_voted_round.cmp(&other.last_voted_round);
        if last_voted_round != Ordering::Equal {
            return last_voted_round;
        }

        self.preferred_round.cmp(&other.preferred_round)
    }
}

impl PersistentStorage for InMemoryStorage {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn set_epoch(&mut self, epoch: u64) -> Result<()> {
        self.epoch = epoch;
        Ok(())
    }

    fn preferred_round(&self) -> Round {
        self.preferred_round
    }

    fn set_preferred_round(&mut self, preferred_round: Round) -> Result<()> {
        self.preferred_round = preferred_round;
        Ok(())
    }

    fn last_voted_round(&self) -> Round {
        self.last_voted_round
    }

    fn set_last_voted_round(&mut self, last_voted_round: Round) -> Result<()> {
        self.last_voted_round = last_voted_round;
        Ok(())
    }
}

#[test]
fn test_in_memory_storage() {
    let mut storage: Box<dyn PersistentStorage> = InMemoryStorage::default_storage();
    assert_eq!(storage.epoch(), 1);
    assert_eq!(storage.last_voted_round(), 0);
    assert_eq!(storage.preferred_round(), 0);
    storage.set_epoch(9).unwrap();
    storage.set_last_voted_round(8).unwrap();
    storage.set_preferred_round(1).unwrap();
    assert_eq!(storage.epoch(), 9);
    assert_eq!(storage.last_voted_round(), 8);
    assert_eq!(storage.preferred_round(), 1);
}

pub struct OnDiskStorage {
    file_path: PathBuf,
    file_path_alt: PathBuf,
    internal_data: InMemoryStorage,
}

impl OnDiskStorage {
    pub fn new_storage(file_path: PathBuf) -> Result<Box<dyn PersistentStorage>> {
        Self::new_internal(file_path, false)
    }

    pub fn default_storage(file_path: PathBuf) -> Result<Box<dyn PersistentStorage>> {
        Self::new_internal(file_path, true)
    }

    fn new_internal(mut file_path: PathBuf, default: bool) -> Result<Box<dyn PersistentStorage>> {
        let mut file_path_alt = PathBuf::from(format!("{}.alt", file_path.to_str().unwrap()));
        let internal_data = InMemoryStorage::load_config(&file_path);
        let internal_data_alt = InMemoryStorage::load_config(&file_path_alt);

        if !default && internal_data.is_err() && internal_data_alt.is_err() {
            if let Err(err) = internal_data {
                return Err(err);
            }
        }

        let mut internal_data = internal_data.unwrap_or_default();
        let internal_data_alt = internal_data_alt.unwrap_or_default();

        if internal_data < internal_data_alt {
            internal_data = internal_data_alt;
            std::mem::swap(&mut file_path, &mut file_path_alt);
        }

        Ok(Box::new(Self {
            file_path,
            file_path_alt,
            internal_data,
        }))
    }

    fn save_and_swap(&mut self) -> Result<()> {
        self.internal_data.save_config(&self.file_path)?;
        std::mem::swap(&mut self.file_path, &mut self.file_path_alt);
        Ok(())
    }
}

impl PersistentStorage for OnDiskStorage {
    fn epoch(&self) -> u64 {
        self.internal_data.epoch()
    }

    fn set_epoch(&mut self, epoch: u64) -> Result<()> {
        self.internal_data.set_epoch(epoch)?;
        self.save_and_swap()?;
        Ok(())
    }

    fn preferred_round(&self) -> Round {
        self.internal_data.preferred_round()
    }

    fn set_preferred_round(&mut self, preferred_round: Round) -> Result<()> {
        self.internal_data.set_preferred_round(preferred_round)?;
        self.save_and_swap()?;
        Ok(())
    }

    fn last_voted_round(&self) -> Round {
        self.internal_data.last_voted_round()
    }

    fn set_last_voted_round(&mut self, last_voted_round: Round) -> Result<()> {
        self.internal_data.set_last_voted_round(last_voted_round)?;
        self.save_and_swap()?;
        Ok(())
    }
}

#[test]
fn test_on_disk_storage() {
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let mut storage: Box<dyn PersistentStorage> =
        OnDiskStorage::default_storage(file_path.clone()).unwrap();
    assert_eq!(storage.epoch(), 1);
    assert_eq!(storage.last_voted_round(), 0);
    assert_eq!(storage.preferred_round(), 0);
    storage.set_epoch(9).unwrap();
    storage.set_last_voted_round(8).unwrap();
    storage.set_preferred_round(1).unwrap();
    assert_eq!(storage.epoch(), 9);
    assert_eq!(storage.last_voted_round(), 8);
    assert_eq!(storage.preferred_round(), 1);

    let storage: Box<dyn PersistentStorage> = OnDiskStorage::default_storage(file_path).unwrap();
    assert_eq!(storage.epoch(), 9);
    assert_eq!(storage.last_voted_round(), 8);
    assert_eq!(storage.preferred_round(), 1);
}
