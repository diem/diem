// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::HashValue,
};
use libra_types::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(content = "value", rename_all = "snake_case", tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Value {
    Ed25519PrivateKey(Ed25519PrivateKey),
    Ed25519PublicKey(Ed25519PublicKey),
    HashValue(HashValue),
    String(String),
    Transaction(Transaction),
    U64(u64),
    Bytes(Vec<u8>),
}

impl Value {
    pub fn ed25519_private_key(self) -> Result<Ed25519PrivateKey, Error> {
        if let Value::Ed25519PrivateKey(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }

    pub fn ed25519_public_key(self) -> Result<Ed25519PublicKey, Error> {
        if let Value::Ed25519PublicKey(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }

    pub fn hash_value(self) -> Result<HashValue, Error> {
        if let Value::HashValue(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }

    pub fn string(self) -> Result<String, Error> {
        if let Value::String(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }

    pub fn u64(self) -> Result<u64, Error> {
        if let Value::U64(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }

    pub fn transaction(self) -> Result<Transaction, Error> {
        if let Value::Transaction(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }

    pub fn bytes(self) -> Result<Vec<u8>, Error> {
        if let Value::Bytes(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }
}
