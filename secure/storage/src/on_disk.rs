// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error, kv_storage::KVStorage, policy::Policy, value::Value, CryptoKVStorage, Storage,
};
use libra_temppath::TempPath;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

/// OnDiskStorage represents a key value store that is persisted to the local filesystem and is
/// intended for single threads (or must be wrapped by a Arc<RwLock<>>). This provides no permission
/// checks and simply offers a proof of concept to unblock building of applications without more
/// complex data stores. Internally, it reads and writes all data to a file, which means that it
/// must make copies of all key material which violates the Libra code base. It violates it because
/// the anticipation is that data stores would securely handle key material. This should not be used
/// in production.
pub struct OnDiskStorage {
    file_path: PathBuf,
    temp_path: TempPath,
}

impl OnDiskStorage {
    pub fn new(file_path: PathBuf) -> Self {
        if !file_path.exists() {
            File::create(&file_path).expect("Unable to create storage");
        }

        // The parent will be one when only a filename is supplied. Therefore use the current
        // working directory provided by PathBuf::new().
        let file_dir = file_path
            .parent()
            .map_or(PathBuf::new(), |p| p.to_path_buf());

        Self {
            file_path,
            temp_path: TempPath::new_with_temp_dir(file_dir),
        }
    }

    fn read(&self) -> Result<HashMap<String, Value>, Error> {
        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data = toml::from_str(&contents)?;
        Ok(data)
    }

    fn write(&self, data: &HashMap<String, Value>) -> Result<(), Error> {
        let contents = toml::to_vec(data)?;
        let mut file = File::create(self.temp_path.path())?;
        file.write_all(&contents)?;
        fs::rename(&self.temp_path, &self.file_path)?;
        Ok(())
    }

    /// Public convenience function to return a new OnDiskStorage based Storage.
    pub fn new_storage(path_buf: PathBuf) -> Box<dyn Storage> {
        Box::new(OnDiskStorage::new(path_buf))
    }
}

impl KVStorage for OnDiskStorage {
    fn available(&self) -> bool {
        true
    }

    fn create(&mut self, key: &str, value: Value, _policy: &Policy) -> Result<(), Error> {
        let mut data = self.read()?;
        if data.contains_key(key) {
            return Err(Error::KeyAlreadyExists(key.to_string()));
        }
        data.insert(key.to_string(), value);
        self.write(&data)
    }

    fn get(&self, key: &str) -> Result<Value, Error> {
        let mut data = self.read()?;
        data.remove(key)
            .ok_or_else(|| Error::KeyNotSet(key.to_string()))
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        let mut data = self.read()?;
        if !data.contains_key(key) {
            return Err(Error::KeyNotSet(key.to_string()));
        }
        data.insert(key.to_string(), value);
        self.write(&data)
    }

    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.write(&HashMap::new())
    }
}

impl CryptoKVStorage for OnDiskStorage {}
