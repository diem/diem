// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, permissions::Permissions, value::Value};

/// Libra interface into storage. Create takes a set of permissions that are enforced internally by
/// the actual backend. The permissions contain public identities that the backend can translate
/// into a unique and private token for another service. Hence get and set internally will pass the
/// current service private token to the backend to gain its permissions.
pub trait Storage {
    /// Creates a new value if it does not exist fails only if there is some other issue.
    fn create_if_not_exists(
        &mut self,
        key: &str,
        value: Value,
        permissions: &Permissions,
    ) -> Result<(), Error> {
        self.create(key, value, permissions).or_else(|err| {
            if let Error::KeyAlreadyExists(_) = err {
                Ok(())
            } else {
                Err(err)
            }
        })
    }
    /// Creates a new value in storage and fails if it already exists
    fn create(&mut self, key: &str, value: Value, permissions: &Permissions) -> Result<(), Error>;
    /// Retreives a value from storage and fails if invalid permiossions or it does not exist
    fn get(&self, key: &str) -> Result<Value, Error>;
    /// Sets a value in storage and fails if invalid permissions or it does not exist
    fn set(&mut self, key: &str, value: Value) -> Result<(), Error>;
}
