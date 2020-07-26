// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, secure_backend::SecureBackend};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_secure_storage::{CryptoStorage, KVStorage, Storage, Value};
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, SignedTransaction, Transaction},
    waypoint::Waypoint,
};
use std::str::FromStr;

/// A helper to handle common error handling and functionality for tooling
pub struct StorageWrapper {
    pub(crate) storage_name: &'static str,
    pub(crate) storage: Storage,
}

impl StorageWrapper {
    pub fn new(
        storage_name: &'static str,
        backend: &SecureBackend,
    ) -> Result<StorageWrapper, Error> {
        let storage = backend.clone().create_storage(storage_name)?;
        Ok(StorageWrapper {
            storage_name,
            storage,
        })
    }

    pub fn new_with_namespace(
        storage_name: &'static str,
        namespace: String,
        backend: &SecureBackend,
    ) -> Result<StorageWrapper, Error> {
        let storage = backend
            .clone()
            .set_namespace(namespace)
            .create_storage(storage_name)?;
        Ok(StorageWrapper {
            storage_name,
            storage,
        })
    }

    pub fn value(&self, name: &'static str) -> Result<Value, Error> {
        self.storage
            .get(name)
            .map(|v| v.value)
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))
    }

    pub fn account_address(&self, name: &'static str) -> Result<AccountAddress, Error> {
        let value = self.string(name)?;
        AccountAddress::from_str(&value).map_err(|e| Error::BackendParsingError(e.to_string()))
    }

    pub fn string(&self, name: &'static str) -> Result<String, Error> {
        self.value(name)?
            .string()
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))
    }

    pub fn transaction(&self, name: &'static str) -> Result<Transaction, Error> {
        self.value(name)?
            .transaction()
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))
    }

    pub fn u64(&self, name: &'static str) -> Result<u64, Error> {
        self.value(name)?
            .u64()
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))
    }

    pub fn waypoint(&self, name: &'static str) -> Result<Waypoint, Error> {
        let actual_waypoint = self.string(name)?;
        Waypoint::from_str(&actual_waypoint).map_err(|e| {
            Error::UnexpectedError(format!("Unable to parse waypoint: {}", e.to_string()))
        })
    }

    /// Retrieves the Public key, that is stored as a public key
    pub fn ed25519_key(&self, name: &'static str) -> Result<Ed25519PublicKey, Error> {
        self.storage
            .get(name)
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))?
            .value
            .ed25519_public_key()
            .map_err(|e| Error::StorageReadError(self.storage_name, name, e.to_string()))
    }

    /// Retrieves the Public key that is stored as a public key
    pub fn x25519_key(&self, name: &'static str) -> Result<x25519::PublicKey, Error> {
        let key = self.ed25519_key(name)?;
        to_x25519(key)
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

    /// Retrieves public key from the stored private key
    pub fn x25519_public_from_private(
        &self,
        key_name: &'static str,
    ) -> Result<x25519::PublicKey, Error> {
        let key = self.ed25519_public_from_private(key_name)?;
        to_x25519(key)
    }

    pub fn set(&mut self, name: &'static str, value: Value) -> Result<(), Error> {
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
