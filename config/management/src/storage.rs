// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    x25519,
};
use libra_network_address_encryption::Encryptor;
use libra_secure_storage::{CryptoStorage, KVStorage, Storage};
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, SignedTransaction, Transaction},
    waypoint::Waypoint,
};
use serde::{de::DeserializeOwned, Serialize};

/// A helper to handle common error handling and functionality for tooling
pub struct StorageWrapper {
    pub(crate) storage_name: &'static str,
    pub(crate) storage: Storage,
}

impl StorageWrapper {
    pub fn encryptor(self) -> Encryptor {
        Encryptor::new(self.storage)
    }

    pub fn value<T: DeserializeOwned>(&self, name: &'static str) -> Result<T, Error> {
        self.storage
            .get(name)
            .map(|v| v.value)
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))
    }

    pub fn account_address(&self, name: &'static str) -> Result<AccountAddress, Error> {
        self.value(name)
    }

    pub fn string(&self, name: &'static str) -> Result<String, Error> {
        self.value(name)
    }

    pub fn transaction(&self, name: &'static str) -> Result<Transaction, Error> {
        self.value(name)
    }

    pub fn u64(&self, name: &'static str) -> Result<u64, Error> {
        self.value(name)
    }

    pub fn waypoint(&self, name: &'static str) -> Result<Waypoint, Error> {
        self.value(name)
    }

    /// Retrieves the Public key, that is stored as a public key
    pub fn ed25519_key(&self, name: &'static str) -> Result<Ed25519PublicKey, Error> {
        self.value(name)
    }

    /// Retrieves the Public key that is stored as a public key
    pub fn x25519_key(&self, name: &'static str) -> Result<x25519::PublicKey, Error> {
        self.ed25519_key(name).and_then(to_x25519)
    }

    pub fn rotate_key(&mut self, name: &'static str) -> Result<Ed25519PublicKey, Error> {
        self.storage
            .rotate_key(name)
            .map_err(|e| Error::StorageWriteError(self.storage_name, name, e.to_string()))
    }

    /// Retrieves public key from the stored private key
    pub fn ed25519_public_from_private(
        &self,
        key_name: &'static str,
    ) -> Result<Ed25519PublicKey, Error> {
        Ok(self
            .storage
            .get_public_key(key_name)
            .map_err(|e| Error::StorageReadError(self.storage_name, key_name, e.to_string()))?
            .public_key)
    }

    /// Retrieves the previous public key from the stored private key
    pub fn ed25519_public_from_private_previous_version(
        &self,
        key_name: &'static str,
    ) -> Result<Ed25519PublicKey, Error> {
        Ok(self
            .storage
            .get_public_key_previous_version(key_name)
            .map_err(|e| Error::StorageReadError(self.storage_name, key_name, e.to_string()))?)
    }

    /// Retrieves public key from the stored private key
    pub fn ed25519_private(&self, key_name: &'static str) -> Result<Ed25519PrivateKey, Error> {
        self.storage
            .export_private_key(key_name)
            .map_err(|e| Error::StorageReadError(self.storage_name, key_name, e.to_string()))
    }

    /// Retrieves public key from the stored private key
    pub fn x25519_public_from_private(
        &self,
        key_name: &'static str,
    ) -> Result<x25519::PublicKey, Error> {
        let key = self.ed25519_public_from_private(key_name)?;
        to_x25519(key)
    }

    pub fn set<T: Serialize>(&mut self, name: &'static str, value: T) -> Result<(), Error> {
        self.storage
            .set(name, value)
            .map_err(|e| Error::StorageWriteError(self.storage_name, name, e.to_string()))
    }

    /// Sign a transaction
    pub fn sign(
        &mut self,
        key_name: &'static str,
        script_name: &'static str,
        raw_transaction: RawTransaction,
    ) -> Result<SignedTransaction, Error> {
        let public_key = self.ed25519_public_from_private(key_name)?;
        let signature = self.storage.sign(key_name, &raw_transaction).map_err(|e| {
            Error::StorageSigningError(self.storage_name, script_name, key_name, e.to_string())
        })?;

        Ok(SignedTransaction::new(
            raw_transaction,
            public_key,
            signature,
        ))
    }

    /// Sign a transaction with the given version
    pub fn sign_using_version(
        &mut self,
        key_name: &'static str,
        key_version: Ed25519PublicKey,
        script_name: &'static str,
        raw_transaction: RawTransaction,
    ) -> Result<SignedTransaction, Error> {
        let signature = self
            .storage
            .sign_using_version(key_name, key_version.clone(), &raw_transaction)
            .map_err(|e| {
                Error::StorageSigningError(self.storage_name, script_name, key_name, e.to_string())
            })?;

        Ok(SignedTransaction::new(
            raw_transaction,
            key_version,
            signature,
        ))
    }
}

pub fn to_x25519(edkey: Ed25519PublicKey) -> Result<x25519::PublicKey, Error> {
    x25519::PublicKey::from_ed25519_public_bytes(&edkey.to_bytes())
        .map_err(|e| Error::UnexpectedError(e.to_string()))
}
