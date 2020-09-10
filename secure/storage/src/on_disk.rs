// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoKVStorage, Error, GetResponse, KVStorage};
use libra_secure_time::{RealTimeService, TimeService};
use libra_temppath::TempPath;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
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
pub type OnDiskStorage = OnDiskStorageInternal<RealTimeService>;

pub struct OnDiskStorageInternal<T> {
    file_path: PathBuf,
    temp_path: TempPath,
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

        // The parent will be one when only a filename is supplied. Therefore use the current
        // working directory provided by PathBuf::new().
        let file_dir = file_path
            .parent()
            .map_or(PathBuf::new(), |p| p.to_path_buf());

        Self {
            file_path,
            temp_path: TempPath::new_with_temp_dir(file_dir),
            time_service,
        }
    }

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

    fn write(&self, data: &HashMap<String, Value>) -> Result<(), Error> {
        let contents = serde_json::to_vec(data)?;
        let mut file = File::create(self.temp_path.path())?;
        file.write_all(&contents)?;
        fs::rename(&self.temp_path, &self.file_path)?;
        Ok(())
    }
}

impl<T: TimeService> KVStorage for OnDiskStorageInternal<T> {
    fn available(&self) -> Result<(), Error> {
        Ok(())
    }

    fn get<V: DeserializeOwned>(&self, key: &str) -> Result<GetResponse<V>, Error> {
        let mut data = self.read()?;
        data.remove(key)
            .ok_or_else(|| Error::KeyNotSet(key.to_string()))
            .and_then(|value| serde_json::from_value(value).map_err(|e| e.into()))
    }

    fn set<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), Error> {
        let mut data = self.read()?;
        data.insert(
            key.to_string(),
            serde_json::to_value(&GetResponse::new(value, self.time_service.now()))?,
        );
        self.write(&data)
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.write(&HashMap::new())
    }
}

impl<T: TimeService> CryptoKVStorage for OnDiskStorageInternal<T> {}
