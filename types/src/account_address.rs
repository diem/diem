// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::authenticator::AuthenticationKey;
use anyhow::{ensure, Error, Result};
use bytes::Bytes;
use libra_crypto::{
    ed25519::Ed25519PublicKey,
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{rngs::OsRng, Rng};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, fmt, str::FromStr};

const SHORT_STRING_LENGTH: usize = 4;

/// A struct that represents an account address.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, CryptoHasher)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountAddress([u8; AccountAddress::LENGTH]);

impl AccountAddress {
    pub const fn new(address: [u8; Self::LENGTH]) -> Self {
        AccountAddress(address)
    }

    /// The number of bytes in an address.
    pub const LENGTH: usize = 16;

    pub const DEFAULT: Self = Self([0u8; AccountAddress::LENGTH]);

    pub fn random() -> Self {
        let mut rng = OsRng;
        let buf: [u8; Self::LENGTH] = rng.gen();
        AccountAddress::new(buf)
    }

    // Helpful in log messages
    pub fn short_str(&self) -> String {
        hex::encode(&self.0[..SHORT_STRING_LENGTH])
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn authentication_key(public_key: &Ed25519PublicKey) -> AuthenticationKey {
        AuthenticationKey::ed25519(public_key)
    }

    pub fn from_public_key(public_key: &Ed25519PublicKey) -> Self {
        AccountAddress::authentication_key(public_key).derived_address()
    }

    pub fn from_hex_literal(literal: &str) -> Result<Self> {
        ensure!(literal.starts_with("0x"), "literal must start with 0x.");

        let hex_len = literal.len() - 2;
        let mut result = if hex_len % 2 != 0 {
            let mut hex_str = String::with_capacity(hex_len + 1);
            hex_str.push('0');
            hex_str.push_str(&literal[2..]);
            hex::decode(&hex_str)?
        } else {
            hex::decode(&literal[2..])?
        };

        let len = result.len();
        let padded_result = if len < Self::LENGTH {
            let mut padded = Vec::with_capacity(Self::LENGTH);
            padded.resize(Self::LENGTH - len, 0u8);
            padded.append(&mut result);
            padded
        } else {
            result
        };

        AccountAddress::try_from(padded_result)
    }
}

impl Default for AccountAddress {
    fn default() -> AccountAddress {
        AccountAddress::DEFAULT
    }
}

impl CryptoHash for AccountAddress {
    type Hasher = AccountAddressHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&self.0);
        state.finish()
    }
}

impl AsRef<[u8]> for AccountAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}

impl fmt::Debug for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}

impl fmt::LowerHex for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl TryFrom<&[u8]> for AccountAddress {
    type Error = Error;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8]) -> Result<AccountAddress> {
        ensure!(
            bytes.len() == Self::LENGTH,
            "The Address {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; Self::LENGTH];
        addr.copy_from_slice(bytes);
        Ok(AccountAddress(addr))
    }
}

impl TryFrom<&[u8; AccountAddress::LENGTH]> for AccountAddress {
    type Error = Error;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8; Self::LENGTH]) -> Result<AccountAddress> {
        AccountAddress::try_from(&bytes[..])
    }
}

impl TryFrom<Vec<u8>> for AccountAddress {
    type Error = Error;

    /// Tries to convert the provided byte buffer into Address.
    fn try_from(bytes: Vec<u8>) -> Result<AccountAddress> {
        AccountAddress::try_from(&bytes[..])
    }
}

impl From<AccountAddress> for Vec<u8> {
    fn from(addr: AccountAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<&AccountAddress> for Vec<u8> {
    fn from(addr: &AccountAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<AccountAddress> for [u8; AccountAddress::LENGTH] {
    fn from(addr: AccountAddress) -> Self {
        addr.0
    }
}

impl From<&AccountAddress> for [u8; AccountAddress::LENGTH] {
    fn from(addr: &AccountAddress) -> Self {
        addr.0
    }
}

impl TryFrom<Bytes> for AccountAddress {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<AccountAddress> {
        AccountAddress::try_from(bytes.as_ref())
    }
}

impl From<AccountAddress> for Bytes {
    fn from(addr: AccountAddress) -> Bytes {
        Bytes::copy_from_slice(addr.0.as_ref())
    }
}

impl From<&AccountAddress> for String {
    fn from(addr: &AccountAddress) -> String {
        ::hex::encode(addr.as_ref())
    }
}

impl TryFrom<String> for AccountAddress {
    type Error = Error;

    fn try_from(s: String) -> Result<AccountAddress> {
        assert!(!s.is_empty());
        let bytes_out = ::hex::decode(s)?;
        AccountAddress::try_from(bytes_out.as_slice())
    }
}

impl FromStr for AccountAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        assert!(!s.is_empty());
        let bytes_out = ::hex::decode(s)?;
        AccountAddress::try_from(bytes_out.as_slice())
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            AccountAddress::try_from(s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "AccountAddress")]
            struct Value([u8; AccountAddress::LENGTH]);

            let value = Value::deserialize(deserializer)?;
            Ok(AccountAddress::new(value.0))
        }
    }
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("AccountAddress", &self.0)
        }
    }
}
