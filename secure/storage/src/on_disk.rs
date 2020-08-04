// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoKVStorage, Error, GetResponse, KVStorage};
use libra_secure_time::{RealTimeService, TimeService};
use named_lock::NamedLock;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

/// OnDiskStorage represents a key value store that is persisted to the local filesystem. Named
/// locks are used to manage concurrent accesses to the storage file:
/// https://crates.io/crates/named-lock
///
/// OnDiskStorage provides no permission checks and simply offers a proof of concept to unblock
/// building of applications without more complex data stores. Internally, it reads and writes all
/// data to a file, which means that it must make copies of all key material which violates the
/// Libra code base. It violates it because the anticipation is that data stores would securely
/// handle key material. This should not be used in production.
pub type OnDiskStorage = OnDiskStorageInternal<RealTimeService>;

pub struct OnDiskStorageInternal<T> {
    file_path: PathBuf,
    lock_file_name: String,
    time_service: T,
}

impl OnDiskStorageInternal<RealTimeService> {
    pub fn new(file_path: PathBuf) -> Self {
        Self::new_with_time_service(file_path, RealTimeService::new())
    }
}

impl<T: TimeService> OnDiskStorageInternal<T> {
    fn new_with_time_service(file_path: PathBuf, time_service: T) -> Self {
        if !file_path.exists() {
            File::create(&file_path).expect("Unable to create storage");
        }

        // We use the file_path to deterministically derive the name of the lock file. This is so
        // that processes sharing a single on_disk file storage will be forced to synchronize on
        // the same lock file. Other on_disk file storages (at different locations) should not
        // be forced to synchronize (e.g., in the case a multi-node swarm has been deployed locally).
        let lock_file_name = file_path
            .to_str()
            .expect("Unable to get the storage file name")
            .to_owned()
            + "_lock_file";

        Self {
            lock_file_name,
            file_path,
            time_service,
        }
    }

    // To ensure safe, concurrent accesses, this method should only be called from get() below.
    fn read(&self) -> Result<HashMap<String, Value>, Error> {
        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.is_empty() {
            return Ok(HashMap::new());
        }
        let data = serde_json::from_str(&contents)?;
        Ok(data)
    }

    // To ensure safe, concurrent accesses, this method should only be called from set() below.
    fn write(&self, data: &HashMap<String, Value>) -> Result<(), Error> {
        let contents = serde_json::to_vec(data)?;

        // Create the file (or overwrite the file if it exists) and write the new file contents.
        let mut file = File::create(&self.file_path)?;
        file.write_all(&contents)?;

        // Force the writes to be synced to the file system (to prevent race conditions
        // where immediate reads don't pick up the new file contents until the writes are synced).
        file.sync_data()?;

        Ok(())
    }

    fn create_lock_file(&self) -> NamedLock {
        NamedLock::create(&self.lock_file_name).expect("Unable to create lock file")
    }
}

impl<T: TimeService> KVStorage for OnDiskStorageInternal<T> {
    fn available(&self) -> Result<(), Error> {
        Ok(())
    }

    fn get<V: DeserializeOwned>(&self, key: &str) -> Result<GetResponse<V>, Error> {
        // Open and lock the file
        let lock = self.create_lock_file();
        lock.lock().expect("Unable to lock the lock file");

        // Read the data from the file
        let mut data = self.read()?;

        // File is unlocked automatically when dropped
        data.remove(key)
            .ok_or_else(|| Error::KeyNotSet(key.to_string()))
            .and_then(|value| serde_json::from_value(value).map_err(|e| e.into()))
    }

    fn set<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), Error> {
        // Open and lock the file
        let lock = self.create_lock_file();
        lock.lock().expect("Unable to lock the lock file");

        // Update the data in the file
        let mut data = self.read()?;
        data.insert(
            key.to_string(),
            serde_json::to_value(&GetResponse::new(value, self.time_service.now()))?,
        );

        // File is unlocked automatically when dropped
        self.write(&data)
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        // Open and lock the file
        let lock = self.create_lock_file();
        lock.lock().expect("Unable to lock the lock file");

        // File is unlocked automatically when dropped
        self.write(&HashMap::new())
    }
}

impl<T: TimeService> CryptoKVStorage for OnDiskStorageInternal<T> {}
