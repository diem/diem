// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, Error, GetResponse, KVStorage, Policy, PublicKeyResponse, Value};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    HashValue,
};

/// This is the Libra interface into secure storage. Any storage engine implementing this trait
/// should support both key/value operations (e.g., get, set and create) and cryptographic key
/// operations (e.g., generate_key, sign_message and rotate_key).
pub trait Storage: KVStorage + CryptoStorage {}

impl<T> Storage for T where T: KVStorage + CryptoStorage {}

/// This is a hack that allows us to convert from SecureBackend into a useable T: Storage, since
/// Box<dyn Storage> is not T: Storage.
pub struct BoxStorage(pub Box<dyn Storage>);

impl KVStorage for BoxStorage {
    fn available(&self) -> bool {
        self.0.available()
    }

    fn get(&self, key: &str) -> Result<GetResponse, Error> {
        self.0.get(key)
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), Error> {
        self.0.set(key, value)
    }

    #[cfg(any(test, feature = "testing"))]
    fn reset_and_clear(&mut self) -> Result<(), Error> {
        self.0.reset_and_clear()
    }
}

impl CryptoStorage for BoxStorage {
    fn create_key(&mut self, name: &str, policy: &Policy) -> Result<Ed25519PublicKey, Error> {
        self.0.create_key(name, policy)
    }

    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error> {
        self.0.export_private_key(name)
    }

    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error> {
        self.0.export_private_key_for_version(name, version)
    }

    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error> {
        self.0.get_public_key(name)
    }

    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error> {
        self.0.rotate_key(name)
    }

    fn sign_message(&mut self, name: &str, message: &HashValue) -> Result<Ed25519Signature, Error> {
        self.0.sign_message(name, message)
    }

    fn sign_message_using_version(
        &mut self,
        name: &str,
        version: Ed25519PublicKey,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error> {
        self.0.sign_message_using_version(name, version, message)
    }
}
