// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bech32::{Bech32, FromBase32, ToBase32};
use bytes::Bytes;
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use crypto::{
    hash::{AccountAddressHasher, CryptoHash, CryptoHasher},
    HashValue, VerifyingKey,
};
use failure::prelude::*;
use hex;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};

pub const ADDRESS_LENGTH: usize = 32;

const SHORT_STRING_LENGTH: usize = 4;

const LIBRA_NETWORK_ID_SHORT: &str = "lb";

/// A struct that represents an account address.
/// Currently Public Key is used.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Default, Clone, Serialize, Deserialize, Copy)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub fn new(address: [u8; ADDRESS_LENGTH]) -> Self {
        AccountAddress(address)
    }

    pub fn random() -> Self {
        let mut rng = OsRng::new().expect("can't access OsRng");
        let buf: [u8; 32] = rng.gen();
        AccountAddress::new(buf)
    }

    // Helpful in log messages
    pub fn short_str(&self) -> String {
        hex::encode(&self.0[0..SHORT_STRING_LENGTH]).to_string()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_public_key<PublicKey: VerifyingKey>(public_key: &PublicKey) -> Self {
        let hash = *HashValue::from_sha3_256(&public_key.to_bytes()).as_ref();
        AccountAddress::new(hash)
    }

    pub fn from_hex_literal(literal: &str) -> Result<Self> {
        let mut hex_string = String::from(&literal[2..]);
        if hex_string.len() % 2 != 0 {
            hex_string.insert(0, '0');
        }

        let mut result = hex::decode(hex_string.as_str())?;
        let len = result.len();
        if len < 32 {
            result.reverse();
            for _ in len..32 {
                result.push(0);
            }
            result.reverse();
        }

        assert!(result.len() >= 32);
        AccountAddress::try_from(result)
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
    type Error = failure::Error;

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

impl TryFrom<&[u8; 32]> for AccountAddress {
    type Error = failure::Error;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8; 32]) -> Result<AccountAddress> {
        AccountAddress::try_from(&bytes[..])
    }
}

impl TryFrom<Vec<u8>> for AccountAddress {
    type Error = failure::Error;

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
    type Error = failure::Error;

    fn try_from(bytes: Bytes) -> Result<AccountAddress> {
        AccountAddress::try_from(bytes.as_ref())
    }
}

impl From<AccountAddress> for Bytes {
    fn from(addr: AccountAddress) -> Bytes {
        addr.0.as_ref().into()
    }
}

impl FromProto for AccountAddress {
    type ProtoType = Vec<u8>;

    fn from_proto(addr: Self::ProtoType) -> Result<Self> {
        AccountAddress::try_from(&addr[..])
    }
}

impl IntoProto for AccountAddress {
    type ProtoType = Vec<u8>;

    fn into_proto(self) -> Self::ProtoType {
        self.0.to_vec()
    }
}

impl From<&AccountAddress> for String {
    fn from(addr: &AccountAddress) -> String {
        ::hex::encode(addr.as_ref())
    }
}

impl TryFrom<String> for AccountAddress {
    type Error = failure::Error;

    fn try_from(s: String) -> Result<AccountAddress> {
        assert!(!s.is_empty());
        let bytes_out = ::hex::decode(s)?;
        AccountAddress::try_from(bytes_out.as_slice())
    }
}

impl TryFrom<Bech32> for AccountAddress {
    type Error = failure::Error;

    fn try_from(encoded_input: Bech32) -> Result<AccountAddress> {
        let base32_hash = encoded_input.data();
        let hash = Vec::from_base32(&base32_hash)?;
        AccountAddress::try_from(&hash[..])
    }
}

impl FromStr for AccountAddress {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        assert!(!s.is_empty());
        let bytes_out = ::hex::decode(s)?;
        AccountAddress::try_from(bytes_out.as_slice())
    }
}

impl TryFrom<AccountAddress> for Bech32 {
    type Error = failure::Error;

    fn try_from(addr: AccountAddress) -> Result<Bech32> {
        let base32_hash = addr.0.to_base32();
        bech32::Bech32::new(LIBRA_NETWORK_ID_SHORT.into(), base32_hash).map_err(Into::into)
    }
}

impl CanonicalSerialize for AccountAddress {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_bytes(&self.0)?;
        Ok(())
    }
}

impl CanonicalDeserialize for AccountAddress {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let bytes = deserializer.decode_bytes()?;
        Self::try_from(bytes)
    }
}
