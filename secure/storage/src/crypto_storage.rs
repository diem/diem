// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use enum_dispatch::enum_dispatch;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    HashValue,
};
use serde::{Deserialize, Serialize};

/// CryptoStorage provides an abstraction for secure generation and handling of cryptographic keys.
#[enum_dispatch]
pub trait CryptoStorage: Send + Sync {
    /// Securely generates a new named Ed25519 private key. The behavior for calling this interface
    /// multiple times with the same name is implementation specific.
    fn create_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Returns the Ed25519 private key stored at 'name'.
    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error>;

    /// An optional API that allows importing private keys and storing them at the provided name.
    /// This is not intended to be used in production and the API may throw unimplemented if
    /// not used correctly. As this is purely a testing API, there is no defined behavior for
    /// importing a key for a given name if that name already exists.  It only exists to allow
    /// Libra to be run in test environments where a set of deterministic keys must be generated.
    fn import_private_key(&mut self, _name: &str, _key: Ed25519PrivateKey) -> Result<(), Error> {
        unimplemented!();
    }

    /// Returns the Ed25519 private key stored at 'name' and identified by 'version', which is the
    /// corresponding public key. This may fail even if the 'named' key exists but the version is
    /// not present.
    fn export_private_key_for_version(
        &self,
        name: &str,
        version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error>;

    /// Returns the Ed25519 public key stored at 'name'.
    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error>;

    /// Rotates an Ed25519 private key. Future calls without version to this 'named' key will
    /// return the rotated key instance. The previous key is retained and can be accessed via
    /// the version. At most two versions are expected to be retained.
    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Signs the provided message using the 'named' private key.
    fn sign_message(&mut self, name: &str, message: &HashValue) -> Result<Ed25519Signature, Error>;

    /// Signs the provided message using the 'named' and 'versioned' private key. This may fail
    /// even if the 'named' key exists but the version is not present.
    fn sign_message_using_version(
        &mut self,
        name: &str,
        version: Ed25519PublicKey,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error>;
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "data")]
pub struct PublicKeyResponse {
    /// Time since Unix Epoch in seconds.
    pub last_update: u64,
    /// Ed25519PublicKey stored at the provided key
    pub public_key: Ed25519PublicKey,
}
