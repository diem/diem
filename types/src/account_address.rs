// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Error, Result};
use bytes::Bytes;
use libra_crypto::{
    ed25519::Ed25519PublicKey,
    hash::{CryptoHash, CryptoHasher},
    multi_ed25519::MultiEd25519PublicKey,
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{rngs::OsRng, Rng};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, fmt, str::FromStr};

pub const ADDRESS_LENGTH: usize = 16;
pub const AUTHENTICATION_KEY_LENGTH: usize = ADDRESS_LENGTH * 2;

const SHORT_STRING_LENGTH: usize = 4;

/// A struct that represents an account address.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, CryptoHasher)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub const fn new(address: [u8; ADDRESS_LENGTH]) -> Self {
        AccountAddress(address)
    }

    pub const DEFAULT: Self = Self([0u8; ADDRESS_LENGTH]);

    pub fn random() -> Self {
        let mut rng = OsRng::new().expect("can't access OsRng");
        let buf: [u8; ADDRESS_LENGTH] = rng.gen();
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
        AuthenticationKey::from_public_key(public_key)
    }

    pub fn from_public_key(public_key: &Ed25519PublicKey) -> Self {
        AccountAddress::authentication_key(public_key).derived_address()
    }

    pub fn from_multisig_key(public_key: &MultiEd25519PublicKey) -> Self {
        AuthenticationKey::from_multisig_key(public_key).derived_address()
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
        let padded_result = if len < ADDRESS_LENGTH {
            let mut padded = Vec::with_capacity(ADDRESS_LENGTH);
            padded.resize(ADDRESS_LENGTH - len, 0u8);
            padded.append(&mut result);
            padded
        } else {
            result
        };

        AccountAddress::try_from(padded_result)
    }
}

/// A struct that represents an account authentication key. An account's address is the last 16
/// bytes of authentication key used to create it
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, CryptoHasher)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AuthenticationKey([u8; AUTHENTICATION_KEY_LENGTH]);

impl AuthenticationKey {
    pub const fn new(bytes: [u8; AUTHENTICATION_KEY_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn random() -> Self {
        let mut rng = OsRng::new().expect("can't access OsRng");
        let buf: [u8; AUTHENTICATION_KEY_LENGTH] = rng.gen();
        AuthenticationKey::new(buf)
    }

    /// Create an authentication key from a single public key by interpreting it as a 1-of-1 of
    /// multikey
    pub fn from_public_key(public_key: &Ed25519PublicKey) -> Self {
        Self::from_public_keys(vec![public_key.clone()])
    }

    /// Create an authentication key from a vector of public keys by interpreting the n-length
    /// vector as a n-of-n multikey
    fn from_public_keys(public_keys: Vec<Ed25519PublicKey>) -> Self {
        let threshold = public_keys.len() as u8;
        Self::from_multisig_key(
            &MultiEd25519PublicKey::new(public_keys, threshold)
                .expect("Failed to create multisig key"),
        )
    }

    /// Create an authentication key from a multikey by computing its SHA3 hash
    pub fn from_multisig_key(public_key: &MultiEd25519PublicKey) -> Self {
        Self(*HashValue::from_sha3_256(&public_key.to_bytes()).as_ref())
    }

    /// Return an address derived from the last ADDRESS_LENGTH bytes of this authentication key
    pub fn derived_address(&self) -> AccountAddress {
        // keep only last 16 bytes
        let mut array = [0u8; ADDRESS_LENGTH];
        array.copy_from_slice(&self.0[AUTHENTICATION_KEY_LENGTH - ADDRESS_LENGTH..]);
        AccountAddress::new(array)
    }

    /// Return the first ADDRESS_LENGTH bytes of this authentication key
    pub fn prefix(&self) -> [u8; ADDRESS_LENGTH] {
        let mut array = [0u8; ADDRESS_LENGTH];
        array.copy_from_slice(&self.0[..ADDRESS_LENGTH]);
        array
    }

    pub fn short_str(&self) -> String {
        hex::encode(&self.0[..SHORT_STRING_LENGTH])
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
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
            bytes.len() == ADDRESS_LENGTH,
            "The Address {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; ADDRESS_LENGTH];
        addr.copy_from_slice(bytes);
        Ok(AccountAddress(addr))
    }
}

impl TryFrom<&[u8; ADDRESS_LENGTH]> for AccountAddress {
    type Error = Error;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8; ADDRESS_LENGTH]) -> Result<AccountAddress> {
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
            let s = <&str>::deserialize(deserializer)?;
            AccountAddress::from_str(s).map_err(D::Error::custom)
        } else {
            let b = <[u8; ADDRESS_LENGTH]>::deserialize(deserializer)?;
            Ok(AccountAddress::new(b))
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
            self.0.serialize(serializer)
        }
    }
}

impl TryFrom<&[u8]> for AuthenticationKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<AuthenticationKey> {
        ensure!(
            bytes.len() == AUTHENTICATION_KEY_LENGTH,
            "The authentication key {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; AUTHENTICATION_KEY_LENGTH];
        addr.copy_from_slice(bytes);
        Ok(AuthenticationKey(addr))
    }
}

impl TryFrom<Vec<u8>> for AuthenticationKey {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<AuthenticationKey> {
        AuthenticationKey::try_from(&bytes[..])
    }
}

impl FromStr for AuthenticationKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        assert!(!s.is_empty());
        let bytes_out = ::hex::decode(s)?;
        AuthenticationKey::try_from(bytes_out.as_slice())
    }
}

impl AsRef<[u8]> for AuthenticationKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::LowerHex for AuthenticationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Display for AuthenticationKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}
