// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, Error, KVStorage, Policy, PublicKeyResponse, Value};
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
    fn create_key(&mut self, name: &str, policy: &Policy) -> Result<Ed25519PublicKey, Error> {
        // Generate and store the new named key pair
        let (private_key, public_key) = new_ed25519_key_pair()?;
        self.create(name, Value::Ed25519PrivateKey(private_key), policy)?;

        // Set the previous key pair version to be the newly generated key pair. This is useful so
        // that we can also set the appropriate policy permissions on the previous key pair version
        // now, and not have to do it later on a rotation.
        self.create(
            &get_previous_version_name(name),
            Value::Ed25519PrivateKey(self.export_private_key(name)?),
            policy,
        )?;

        Ok(public_key)
    }

    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error> {
        match self.get(name)?.value {
            Value::Ed25519PrivateKey(private_key) => Ok(private_key),
            _ => Err(Error::UnexpectedValueType),
        }
    }

    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        let current_private_key = self.export_private_key(name)?;
        if current_private_key.public_key().eq(&version) {
            return Ok(current_private_key);
        }

        let previous_private_key = self.export_private_key(&get_previous_version_name(name))?;
        if previous_private_key.public_key().eq(&version) {
            return Ok(previous_private_key);
        }

        Err(Error::KeyVersionNotFound(version.to_string()))
    }

    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error> {
        let response = self.get(name)?;

        let public_key = match &response.value {
            Value::Ed25519PrivateKey(private_key) => private_key.public_key(),
            _ => return Err(Error::UnexpectedValueType),
        };

        Ok(PublicKeyResponse {
            last_update: response.last_update,
            public_key,
        })
    }

    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        match self.get(name)?.value {
            Value::Ed25519PrivateKey(private_key) => {
                let (new_private_key, new_public_key) = new_ed25519_key_pair()?;
                self.set(
                    &get_previous_version_name(name),
                    Value::Ed25519PrivateKey(private_key),
                )?;
                self.set(name, Value::Ed25519PrivateKey(new_private_key))?;
                Ok(new_public_key)
            }
            _ => Err(Error::UnexpectedValueType),
        }
    }

    fn sign_message(&mut self, name: &str, message: &HashValue) -> Result<Ed25519Signature, Error> {
        let private_key = self.export_private_key(name)?;
        Ok(private_key.sign_message(message))
    }

    fn sign_message_using_version(
        &mut self,
        name: &str,
        version: Ed25519PublicKey,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error> {
        let private_key = self.export_private_key_for_version(name, version)?;
        Ok(private_key.sign_message(message))
    }
}

/// Private helper method to generate a new ed25519 key pair using entropy from the OS.
fn new_ed25519_key_pair() -> Result<(Ed25519PrivateKey, Ed25519PublicKey), Error> {
    let mut seed_rng = OsRng;
    let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key = private_key.public_key();
    Ok((private_key, public_key))
}

/// Private helper method to get the name of the previous version of the given key pair, as held in
/// secure cryptographic storage.
fn get_previous_version_name(name: &str) -> String {
    format!("{}_previous", name)
}
