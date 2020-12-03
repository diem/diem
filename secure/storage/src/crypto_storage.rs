// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use diem_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

/// CryptoStorage provides an abstraction for secure generation and handling of cryptographic keys.
#[enum_dispatch]
pub trait CryptoStorage {
    /// Securely generates a new named Ed25519 private key. The behavior for calling this interface
    /// multiple times with the same name is implementation specific.
    fn create_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Returns the Ed25519 private key stored at 'name'.
    fn export_private_key(&self, name: &str) -> Result<Ed25519PrivateKey, Error>;

    /// An optional API that allows importing private keys and storing them at the provided name.
    /// This is not intended to be used in production and the API may throw unimplemented if
    /// not used correctly. As this is purely a testing API, there is no defined behavior for
    /// importing a key for a given name if that name already exists.  It only exists to allow
    /// Diem to be run in test environments where a set of deterministic keys must be generated.
    fn import_private_key(&mut self, name: &str, key: Ed25519PrivateKey) -> Result<(), Error>;

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

    /// Returns the previous version of the Ed25519 public key stored at 'name'. For the most recent
    /// version, see 'get_public_key(..)' above.
    fn get_public_key_previous_version(&self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Rotates an Ed25519 private key. Future calls without version to this 'named' key will
    /// return the rotated key instance. The previous key is retained and can be accessed via
    /// the version. At most two versions are expected to be retained.
    fn rotate_key(&mut self, name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Signs the provided securely-hashable struct, using the 'named' private
    /// key.
    // The FQDNs on the next line help macros don't remove them
    fn sign<T: diem_crypto::hash::CryptoHash + serde::Serialize>(
        &self,
        name: &str,
        message: &T,
    ) -> Result<Ed25519Signature, Error>;

    /// Signs the provided securely-hashable struct, using the 'named' and 'versioned' private key. This may fail
    /// even if the 'named' key exists but the version is not present.
    // The FQDNs on the next line help macros, don't remove them
    fn sign_using_version<T: diem_crypto::hash::CryptoHash + serde::Serialize>(
        &self,
        name: &str,
        version: Ed25519PublicKey,
        message: &T,
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
