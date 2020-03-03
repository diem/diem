// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Value};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    HashValue, PrivateKey, SigningKey, Uniform,
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
    /// generation and rotation), should override this implementation.
    fn generate_new_ed25519_key_pair(
        &mut self,
        key_pair_name: &str,
        policy: &Policy,
    ) -> Result<Ed25519PublicKey, Error> {
        // Generate and store the new named key pair
        let (private_key, public_key) = new_ed25519_key_pair()?;
        self.create(key_pair_name, Value::Ed25519PrivateKey(private_key), policy)?;

        // Set the previous key pair version to be the newly generated key pair. This is useful so
        // that we can also set the appropriate policy permissions on the previous key pair version
        // now, and not have to do it later on a rotation.
        self.create(
            &get_previous_version_name(key_pair_name),
            Value::Ed25519PrivateKey(self.get_private_key_for_name(key_pair_name)?),
            policy,
        )?;

        Ok(public_key)
    }

    /// Returns the public key for a given Ed25519 key pair, as identified by the 'key_pair_name'.
    /// If the key pair doesn't exist, or the caller doesn't have the
    /// appropriate permissions to retrieve the public key, this call will fail with an error.
    fn get_public_key_for_name(&self, key_pair_name: &str) -> Result<Ed25519PublicKey, Error> {
        self.get_private_key_for_name(key_pair_name)
            .map(|e| e.public_key())
    }

    /// Returns the private key for a given Ed25519 key pair, as identified by the 'key_pair_name'.
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate permissions to
    /// retrieve the private key, this call will fail with an error.
    fn get_private_key_for_name(&self, key_pair_name: &str) -> Result<Ed25519PrivateKey, Error> {
        match self.get(key_pair_name)? {
            Value::Ed25519PrivateKey(private_key) => Ok(private_key),
            _ => Err(Error::UnexpectedValueType),
        }
    }

    /// Returns the private key for a given Ed25519 key pair version, as identified by the
    /// 'key_pair_name' and 'key_pair_version'. If the key pair at the specified version doesn't
    /// exist, or the caller doesn't have the appropriate permissions to retrieve the private key,
    /// this call will fail with an error.
    fn get_private_key_for_name_and_version(
        &self,
        key_pair_name: &str,
        key_pair_version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        let current_private_key = self.get_private_key_for_name(key_pair_name)?;
        if current_private_key.public_key().eq(&key_pair_version) {
            return Ok(current_private_key);
        }

        let previous_private_key =
            self.get_private_key_for_name(&get_previous_version_name(key_pair_name))?;
        if previous_private_key.public_key().eq(&key_pair_version) {
            return Ok(previous_private_key);
        }

        Err(Error::KeyVersionNotFound(key_pair_version.to_string()))
    }

    /// Rotates an Ed25519 key pair by generating a new Ed25519 key pair, and updating the
    /// 'key_pair_name' to reference the freshly generated key. The previous key pair is retained
    /// in storage if needed. If multiple key rotations occur over the lifetime of a key pair, only
    /// two versions of the key pair are maintained (i.e., the current and previous one).
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate permissions to
    /// retrieve the public key, this call will fail with an error. Otherwise, the new public
    /// key for the rotated key pair is returned.
    fn rotate_key_pair(&mut self, key_pair_name: &str) -> Result<Ed25519PublicKey, Error> {
        match self.get(key_pair_name)? {
            Value::Ed25519PrivateKey(private_key) => {
                let (new_private_key, new_public_key) = new_ed25519_key_pair()?;
                self.set(
                    &get_previous_version_name(key_pair_name),
                    Value::Ed25519PrivateKey(private_key),
                )?;
                self.set(key_pair_name, Value::Ed25519PrivateKey(new_private_key))?;
                Ok(new_public_key)
            }
            _ => Err(Error::UnexpectedValueType),
        }
    }

    /// Signs the given message using the private key associated with the given 'key_pair_name'.
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate
    /// permissions to retrieve and use the public key, this call will fail with an error.
    fn sign_message(
        &mut self,
        key_pair_name: &str,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error> {
        let private_key = self.get_private_key_for_name(key_pair_name)?;
        Ok(private_key.sign_message(message))
    }

    /// Signs the given message using the private key associated with the given 'key_pair_name'
    /// and 'key_pair_version'. If the key pair doesn't exist, or the caller doesn't have the
    /// appropriate permissions to perform the operation, this call will fail with an error.
    /// Note: the 'key_pair_version' is specified using the public key associated with a key pair.
    /// Only two versions of a key pair are ever maintained at any given time (i.e., the
    /// current version, and the previous version).
    fn sign_message_using_version(
        &mut self,
        key_pair_name: &str,
        key_pair_version: Ed25519PublicKey,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error> {
        let private_key =
            self.get_private_key_for_name_and_version(key_pair_name, key_pair_version)?;
        Ok(private_key.sign_message(message))
    }

    /// Resets and clears all data held in the storage engine.
    /// Note: this should only be exposed and used for testing. Resetting the storage engine is not
    /// something that should be supported in production.
    #[cfg(test)]
    fn reset_and_clear(&mut self) -> Result<(), Error>;
}

/// Helper method to generate a new ed25519 key pair using entropy from the OS.
fn new_ed25519_key_pair() -> Result<(Ed25519PrivateKey, Ed25519PublicKey), Error> {
    let mut seed_rng = OsRng::new().map_err(|e| Error::EntropyError(e.to_string()))?;
    let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
    let private_key = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let public_key = private_key.public_key();
    Ok((private_key, public_key))
}

/// Helper method to get the name of the previous version of the given key pair, as held in
/// secure storage.
fn get_previous_version_name(key_pair_name: &str) -> String {
    format!("{}_previous", key_pair_name)
}
