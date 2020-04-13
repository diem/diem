// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy};
use libra_crypto::{ed25519, HashValue};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// CryptoStorage offers a secure storage engine for generating, using and managing cryptographic
/// keys securely. The API offered here is inspired by the 'Transit Secret Engine' provided by
/// Vault: a production-ready storage engine for cloud infrastructures (e.g., see
/// https://www.vaultproject.io/docs/secrets/transit/index.html).
pub trait CryptoStorage: Send + Sync {
    /// Securely generates a new named Ed25519 key pair and returns the corresponding public key.
    ///
    /// The new key pair is named according to the 'name' and is held in secure storage
    /// under the given 'policy'. To access or use the key pair (e.g., sign or encrypt data),
    /// subsequent API calls must refer to the key pair by name. As this API call may fail
    /// (e.g., if a key pair with the given name already exists), an error may also be returned.
    fn create_key(&mut self, name: &str, policy: &Policy) -> Result<ed25519::PublicKey, Error>;

    /// Returns the private key for a given Ed25519 key pair, as identified by the 'name'.
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate permissions to
    /// retrieve the private key, this call will fail with an error.
    fn export_private_key(&self, name: &str) -> Result<ed25519::PrivateKey, Error>;

    /// Returns the private key for a given Ed25519 key pair version, as identified by the
    /// 'name' and 'version'. If the key pair at the specified version doesn't
    /// exist, or the caller doesn't have the appropriate permissions to retrieve the private key,
    /// this call will fail with an error.
    fn export_private_key_for_version(
        &self,
        name: &str,
        version: ed25519::PublicKey,
    ) -> Result<ed25519::PrivateKey, Error>;

    /// Returns the public key for a given Ed25519 key pair, as identified by the 'name'.
    /// If the key pair doesn't exist, or the caller doesn't have the
    /// appropriate permissions to retrieve the public key, this call will fail with an error.
    fn get_public_key(&self, name: &str) -> Result<PublicKeyResponse, Error>;

    /// Rotates an Ed25519 key pair by generating a new Ed25519 key pair, and updating the
    /// 'name' to reference the freshly generated key. The previous key pair is retained
    /// in storage if needed. If multiple key rotations occur over the lifetime of a key pair, only
    /// two versions of the key pair are maintained (i.e., the current and previous one).
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate permissions to
    /// retrieve the public key, this call will fail with an error. Otherwise, the new public
    /// key for the rotated key pair is returned.
    fn rotate_key(&mut self, name: &str) -> Result<ed25519::PublicKey, Error>;

    /// Signs the given message using the private key associated with the given 'name'.
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate
    /// permissions to retrieve and use the public key, this call will fail with an error.
    fn sign_message(
        &mut self,
        name: &str,
        message: &HashValue,
    ) -> Result<ed25519::Signature, Error>;

    /// Signs the given message using the private key associated with the given 'name'
    /// and 'version'. If the key pair doesn't exist, or the caller doesn't have the
    /// appropriate permissions to perform the operation, this call will fail with an error.
    /// Note: the 'version' is specified using the public key associated with a key pair.
    /// Only two versions of a key pair are ever maintained at any given time (i.e., the
    /// current version, and the previous version).
    fn sign_message_using_version(
        &mut self,
        name: &str,
        version: ed25519::PublicKey,
        message: &HashValue,
    ) -> Result<ed25519::Signature, Error>;
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "data")]
pub struct PublicKeyResponse {
    /// Time since Unix Epoch in seconds.
    pub last_update: u64,
    /// ed25519::PublicKey stored at the provided key
    pub public_key: ed25519::PublicKey,
}

impl PublicKeyResponse {
    /// Creates a PublicKeyResponse using the current time for the timestamp
    pub fn new(public_key: ed25519::PublicKey) -> Self {
        Self {
            public_key,
            last_update: Self::now().as_secs(),
        }
    }

    /// Returns back a Duration encompassing the current system time less the Unix Epoch
    fn now() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    }
}
