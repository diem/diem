// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoKVStorage, Error, GetResponse, KVStorage, Value};
use libra_github_client::Client;
use libra_secure_time::{RealTimeService, TimeService};

/// GitHubStorage leverages a GitHub repository to provide a file system approach to key / value
/// storage.  This is not intended for storing private data but for organizing public data.
pub struct GitHubStorage {
    client: Client,
    time_service: RealTimeService,
}

impl GitHubStorage {
    pub fn new(owner: String, repository: String, token: String) -> Self {
        Self {
            client: Client::new(owner, repository, token),
            time_service: RealTimeService::new(),
        }
    }
}

impl KVStorage for GitHubStorage {
    fn available(&self) -> Result<(), Error> {
        if !self.client.get_branches()?.is_empty() {
            Ok(())
        } else {
            Err(Error::InternalError("No branches found.".into()))
        }
    }

    fn get(&self, key: &str) -> Result<GetResponse, Error> {
        let data = self.client.get_file(key)?;
        let data = base64::decode(&data)?;
        let data = std::str::from_utf8(&data).unwrap();
        serde_json::from_str(&data).map_err(|e| e.into())
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        let data = GetResponse::new(value, self.time_service.now());
        let data = serde_json::to_string(&data)?;
        let data = base64::encode(&data);
        self.client.put(key, &data)?;
        Ok(())
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.client.delete_directory("/").map_err(|e| e.into())
    }
}

impl CryptoKVStorage for GitHubStorage {}
