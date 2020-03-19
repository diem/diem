// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Value};

/// A secure key/value storage engine. Create takes a policy that is enforced internally by the
/// actual backend. The policy contains public identities that the backend can translate into a
/// unique and private token for another service. Hence get and set internally will pass the
/// current service private token to the backend to gain its permissions.
pub trait KVStorage: Send + Sync {
    /// Returns true if the backend service is online and available.
    fn available(&self) -> bool;

    /// Creates a new value in storage and fails if it already exists
    fn create(&mut self, key: &str, value: Value, policy: &Policy) -> Result<(), Error>;

    /// Creates a new value if it does not exist fails only if there is some other issue.
    fn create_if_not_exists(
        &mut self,
        key: &str,
        value: Value,
        policy: &Policy,
    ) -> Result<(), Error> {
        self.create(key, value, policy).or_else(|err| {
            if let Error::KeyAlreadyExists(_) = err {
                Ok(())
            } else {
                Err(err)
            }
        })
    }

    /// Retrieves a value from storage and fails if invalid permissions or it does not exist
    fn get(&self, key: &str) -> Result<Value, Error>;

    /// Sets a value in storage and fails if invalid permissions or it does not exist
    fn set(&mut self, key: &str, value: Value) -> Result<(), Error>;

    // @TODO(davidiw): Make this accessible only to tests.
    /// Resets and clears all data held in the storage engine.
    /// Note: this should only be exposed and used for testing. Resetting the storage engine is not
    /// something that should be supported in production.
    fn reset_and_clear(&mut self) -> Result<(), Error>;
}
