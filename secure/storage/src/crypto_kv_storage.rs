// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{kv_storage::KVStorage, CryptoStorage, Error, Policy, Value};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    HashValue, PrivateKey, SigningKey, Uniform,
};
use rand::{rngs::OsRng, Rng, SeedableRng};

/// CryptoKVStorage offers a CryptoStorage implementation by extending a key value store (KVStorage)
/// to create and manage cryptographic keys. This is useful for providing a simple CryptoStorage
/// implementation based upon an existing KVStorage engine (e.g. for test purposes).
pub trait CryptoKVStorage: KVStorage {}

impl<T: CryptoKVStorage> CryptoStorage for T {
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

    fn get_public_key_for_name(&self, key_pair_name: &str) -> Result<Ed25519PublicKey, Error> {
        self.get_private_key_for_name(key_pair_name)
            .map(|e| e.public_key())
    }

    fn get_private_key_for_name(&self, key_pair_name: &str) -> Result<Ed25519PrivateKey, Error> {
        match self.get(key_pair_name)? {
            Value::Ed25519PrivateKey(private_key) => Ok(private_key),
            _ => Err(Error::UnexpectedValueType),
        }
    }

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

    fn sign_message(
        &mut self,
        key_pair_name: &str,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error> {
        let private_key = self.get_private_key_for_name(key_pair_name)?;
        Ok(private_key.sign_message(message))
    }

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
}

/// Private helper method to generate a new ed25519 key pair using entropy from the OS.
fn new_ed25519_key_pair() -> Result<(Ed25519PrivateKey, Ed25519PublicKey), Error> {
    let mut seed_rng = OsRng::new().map_err(|e| Error::EntropyError(e.to_string()))?;
    let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
    let private_key = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let public_key = private_key.public_key();
    Ok((private_key, public_key))
}

/// Private helper method to get the name of the previous version of the given key pair, as held in
/// secure cryptographic storage.
fn get_previous_version_name(key_pair_name: &str) -> String {
    format!("{}_previous", key_pair_name)
}
