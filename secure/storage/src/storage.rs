// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Value};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform,
};
use rand::{rngs::OsRng, Rng, SeedableRng};

/// Libra interface into storage. Create takes a policy that is enforced internally by the actual
/// backend. The policy contains public identities that the backend can translate into a unique and
/// private token for another service. Hence get and set internally will pass the current service
/// private token to the backend to gain its permissions.
/// Storage also offers support for generating, using and managing cryptographic keys securely.
/// The API offered here is inspired by the 'Transit Secret Engine' provided by Vault: a
/// production-ready storage engine for cloud infrastructures (e.g.,
/// see https://www.vaultproject.io/docs/secrets/transit/index.html).
pub trait Storage: Send + Sync {
    /// Returns true if the backend service is online and available.
    fn available(&self) -> bool;
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
    /// Creates a new value in storage and fails if it already exists
    fn create(&mut self, key: &str, value: Value, policy: &Policy) -> Result<(), Error>;
    /// Retreives a value from storage and fails if invalid permiossions or it does not exist
    fn get(&self, key: &str) -> Result<Value, Error>;
    /// Sets a value in storage and fails if invalid permissions or it does not exist
    fn set(&mut self, key: &str, value: Value) -> Result<(), Error>;

    /// Securely generates a new named Ed25519 key pair and returns the corresponding public key.
    ///
    /// The new key pair is named according to the 'key_pair_name' and is held in secure storage
    /// under the given 'policy'. To access or use the key pair (e.g., sign or encrypt data),
    /// subsequent API calls must refer to the key pair by name. As this API call may fail
    /// (e.g., if a key pair with the given name already exists), an error may also be returned.
    ///
    /// This default implementation is useful for storage engines that only implement the simple
    /// Storage interface. More complex engines (e.g., those that support native crypto key
    /// generation), should override this implementation.
    fn generate_new_ed25519_key_pair(
        &mut self,
        key_pair_name: &str,
        policy: &Policy,
    ) -> Result<Ed25519PublicKey, Error> {
        let mut seed_rng = OsRng::new().map_err(|e| Error::EntropyError(e.to_string()))?;
        let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
        let private_key = Ed25519PrivateKey::generate_for_testing(&mut rng);
        let public_key = private_key.public_key();
        self.create(key_pair_name, Value::Ed25519PrivateKey(private_key), policy)
            .map(|_| public_key)
    }

    /// Returns the corresponding public key for a given Ed25519 key pair, as identified by the
    /// given 'key_pair_name'. If the key pair doesn't exist, or the caller doesn't have the
    /// appropriate permissions to retrieve the public key, this call will fail with an error.
    fn get_public_key_for(&mut self, key_pair_name: &str) -> Result<Ed25519PublicKey, Error> {
        match self.get(key_pair_name)? {
            Value::Ed25519PrivateKey(private_key) => Ok(private_key.public_key()),
            _ => Err(Error::UnexpectedValueType),
        }
    }

    /// Resets and clears all data held in the storage engine.
    /// Note: this should only be exposed and used for testing. Resetting the storage engine is not
    /// something that should be supported in production.
    #[cfg(test)]
    fn reset_and_clear(&mut self) -> Result<(), Error>;
}
