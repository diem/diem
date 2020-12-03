// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, PersistentSafetyStorage};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
};
use diem_global_constants::CONSENSUS_KEY;
use diem_types::{account_address::AccountAddress, validator_signer::ValidatorSigner};
use serde::Serialize;

/// A ConfigurableValidatorSigner is a ValidatorSigner wrapper that offers either
/// a ValidatorSigner instance or a ValidatorHandle instance, depending on the
/// configuration chosen. This abstracts away the complexities of handling either
/// instance, while offering the same API as a ValidatorSigner.
pub enum ConfigurableValidatorSigner {
    Signer(ValidatorSigner),
    Handle(ValidatorHandle),
}

impl ConfigurableValidatorSigner {
    /// Returns a new ValidatorSigner instance
    pub fn new_signer(author: AccountAddress, consensus_key: Ed25519PrivateKey) -> Self {
        let signer = ValidatorSigner::new(author, consensus_key);
        ConfigurableValidatorSigner::Signer(signer)
    }

    /// Returns a new ValidatorHandle instance
    pub fn new_handle(author: AccountAddress, key_version: Ed25519PublicKey) -> Self {
        let handle = ValidatorHandle::new(author, key_version);
        ConfigurableValidatorSigner::Handle(handle)
    }

    /// Returns the author associated with the signer configuration.
    pub fn author(&self) -> AccountAddress {
        match self {
            ConfigurableValidatorSigner::Signer(signer) => signer.author(),
            ConfigurableValidatorSigner::Handle(handle) => handle.author(),
        }
    }

    /// Returns the public key associated with the signer configuration.
    pub fn public_key(&self) -> Ed25519PublicKey {
        match self {
            ConfigurableValidatorSigner::Signer(signer) => signer.public_key(),
            ConfigurableValidatorSigner::Handle(handle) => handle.key_version(),
        }
    }

    /// Signs a given message using the signer configuration.
    pub fn sign<T: Serialize + CryptoHash>(
        &self,
        message: &T,
        storage: &PersistentSafetyStorage,
    ) -> Result<Ed25519Signature, Error> {
        match self {
            ConfigurableValidatorSigner::Signer(signer) => Ok(signer.sign(message)),
            ConfigurableValidatorSigner::Handle(handle) => handle.sign(message, storage),
        }
    }
}

/// A ValidatorHandle associates a validator with a consensus key version held in storage.
/// In contrast to a ValidatorSigner, ValidatorHandle does not hold the private
/// key directly but rather holds a reference to that private key which should be
/// accessed using the handle and the secure storage backend.
pub struct ValidatorHandle {
    author: AccountAddress,
    key_version: Ed25519PublicKey,
}

impl ValidatorHandle {
    pub fn new(author: AccountAddress, key_version: Ed25519PublicKey) -> Self {
        ValidatorHandle {
            author,
            key_version,
        }
    }

    /// Returns the author associated with this handle.
    pub fn author(&self) -> AccountAddress {
        self.author
    }

    /// Returns the public key version associated with this handle.
    pub fn key_version(&self) -> Ed25519PublicKey {
        self.key_version.clone()
    }

    /// Signs a given message using this handle and a given secure storage backend.
    pub fn sign<T: Serialize + CryptoHash>(
        &self,
        message: &T,
        storage: &PersistentSafetyStorage,
    ) -> Result<Ed25519Signature, Error> {
        storage.sign(CONSENSUS_KEY.into(), self.key_version(), message)
    }
}
