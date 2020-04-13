// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use libra_crypto::{ed25519, hash::HashValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(content = "value", rename_all = "snake_case", tag = "type")]
pub enum Value {
    Ed25519PrivateKey(ed25519::PrivateKey),
    HashValue(HashValue),
    U64(u64),
}

impl Value {
    pub fn ed25519_private_key(self) -> Result<ed25519::PrivateKey, Error> {
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

    pub fn from_base64(input: &str) -> Result<Value, Error> {
        let bytes = base64::decode(input)?;
        let value = lcs::from_bytes(&bytes)?;
        Ok(value)
    }

    pub fn to_base64(&self) -> Result<String, Error> {
        let bytes = lcs::to_bytes(self)?;
        Ok(base64::encode(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libra_crypto::Uniform;

    #[test]
    fn ed25519_private_key() {
        let value = ed25519::PrivateKey::generate_for_testing();
        let value = Value::Ed25519PrivateKey(value);
        let base64 = value.to_base64().unwrap();
        let out_value = Value::from_base64(&base64).unwrap();
        assert_eq!(value, out_value);
    }

    #[test]
    fn hash_value() {
        let value = Value::HashValue(HashValue::random());
        let base64 = value.to_base64().unwrap();
        let out_value = Value::from_base64(&base64).unwrap();
        assert_eq!(value, out_value);
    }

    #[test]
    fn u64() {
        let value = Value::U64(12341);
        let base64 = value.to_base64().unwrap();
        let out_value = Value::from_base64(&base64).unwrap();
        assert_eq!(value, out_value);
    }
}
