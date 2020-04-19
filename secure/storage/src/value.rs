// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use libra_crypto::{ed25519::Ed25519PrivateKey, hash::HashValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(content = "value", rename_all = "snake_case", tag = "type")]
pub enum Value {
    Ed25519PrivateKey(Ed25519PrivateKey),
    HashValue(HashValue),
    U64(u64),
}

impl Value {
    pub fn ed25519_private_key(self) -> Result<Ed25519PrivateKey, Error> {
        if let Value::Ed25519PrivateKey(value) = self {
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

    pub fn u64(self) -> Result<u64, Error> {
        if let Value::U64(value) = self {
            Ok(value)
        } else {
            Err(Error::UnexpectedValueType)
        }
    }
}
