// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    HashValue,
};

/// CryptoStorage offers a secure storage engine for generating, using and managing cryptographic
/// keys securely. The API offered here is inspired by the 'Transit Secret Engine' provided by
/// Vault: a production-ready storage engine for cloud infrastructures (e.g., see
/// https://www.vaultproject.io/docs/secrets/transit/index.html).
pub trait CryptoStorage: Send + Sync {
    /// Securely generates a new named Ed25519 key pair and returns the corresponding public key.
    ///
    /// The new key pair is named according to the 'key_pair_name' and is held in secure storage
    /// under the given 'policy'. To access or use the key pair (e.g., sign or encrypt data),
    /// subsequent API calls must refer to the key pair by name. As this API call may fail
    /// (e.g., if a key pair with the given name already exists), an error may also be returned.
    fn generate_new_ed25519_key_pair(
        &mut self,
        key_pair_name: &str,
        policy: &Policy,
    ) -> Result<Ed25519PublicKey, Error>;

    /// Returns the public key for a given Ed25519 key pair, as identified by the 'key_pair_name'.
    /// If the key pair doesn't exist, or the caller doesn't have the
    /// appropriate permissions to retrieve the public key, this call will fail with an error.
    fn get_public_key_for_name(&self, key_pair_name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Returns the private key for a given Ed25519 key pair, as identified by the 'key_pair_name'.
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate permissions to
    /// retrieve the private key, this call will fail with an error.
    fn get_private_key_for_name(&self, key_pair_name: &str) -> Result<Ed25519PrivateKey, Error>;

    /// Returns the private key for a given Ed25519 key pair version, as identified by the
    /// 'key_pair_name' and 'key_pair_version'. If the key pair at the specified version doesn't
    /// exist, or the caller doesn't have the appropriate permissions to retrieve the private key,
    /// this call will fail with an error.
    fn get_private_key_for_name_and_version(
        &self,
        key_pair_name: &str,
        key_pair_version: Ed25519PublicKey,
    ) -> Result<Ed25519PrivateKey, Error>;

    /// Rotates an Ed25519 key pair by generating a new Ed25519 key pair, and updating the
    /// 'key_pair_name' to reference the freshly generated key. The previous key pair is retained
    /// in storage if needed. If multiple key rotations occur over the lifetime of a key pair, only
    /// two versions of the key pair are maintained (i.e., the current and previous one).
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate permissions to
    /// retrieve the public key, this call will fail with an error. Otherwise, the new public
    /// key for the rotated key pair is returned.
    fn rotate_key_pair(&mut self, key_pair_name: &str) -> Result<Ed25519PublicKey, Error>;

    /// Signs the given message using the private key associated with the given 'key_pair_name'.
    /// If the key pair doesn't exist, or the caller doesn't have the appropriate
    /// permissions to retrieve and use the public key, this call will fail with an error.
    fn sign_message(
        &mut self,
        key_pair_name: &str,
        message: &HashValue,
    ) -> Result<Ed25519Signature, Error>;

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
    ) -> Result<Ed25519Signature, Error>;
}
