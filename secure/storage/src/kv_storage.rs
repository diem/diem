// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Value};
use serde::{Deserialize, Serialize};

/// A secure key/value storage engine. Create takes a policy that is enforced internally by the
/// actual backend. The policy contains public identities that the backend can translate into a
/// unique and private token for another service. Hence get and set internally will pass the
/// current service private token to the backend to gain its permissions.
pub trait KVStorage: Send + Sync {
    /// Returns an error if the backend service is not online and available.
    fn available(&self) -> Result<(), Error>;

    /// Retrieves a value from storage and fails if the backend is unavailable or the process has
    /// invalid permissions.
    fn get(&self, key: &str) -> Result<GetResponse, Error>;

    /// Sets a value in storage and fails if the backend is unavailable or the process has
    /// invalid permissions.
    fn set(&mut self, key: &str, value: Value) -> Result<(), Error>;

    /// Resets and clears all data held in the storage engine.
    /// Note: this should only be exposed and used for testing. Resetting the storage engine is not
    /// something that should be supported in production.
    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error>;
}

/// A container for a get response that contains relevant metadata and the value stored at the
/// given key.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "data")]
pub struct GetResponse {
    /// Time since Unix Epoch in seconds.
    pub last_update: u64,
    /// Value stored at the provided key
    pub value: Value,
}

impl GetResponse {
    /// Creates a GetResponse
    pub fn new(value: Value, last_update: u64) -> Self {
        Self { value, last_update }
    }
}
