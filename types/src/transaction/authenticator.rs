// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::{ensure, Error, Result};
use libra_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
    traits::Signature,
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};

/// A `TransactionAuthenticator` is an an abstraction of a signature scheme. It must know:
/// (1) How to check its signature against a message and public key
/// (2) How to convert its public key into an `AuthenticationKeyPreimage` structured as
/// (public_key | signaure_scheme_id).
/// Each on-chain `LibraAccount` must store an `AuthenticationKey` (computed via a sha3 hash of an
/// `AuthenticationKeyPreimage`).
/// Each transaction submitted to the Libra blockchain contains a `TransactionAuthenticator`. During
/// transaction execution, the executor will check if the `TransactionAuthenticator`'s signature on
/// the transaction hash is well-formed (1) and whether the sha3 hash of the
/// `TransactionAuthenticator`'s `AuthenticationKeyPreimage` matches the `AuthenticationKey` stored
/// under the transaction's sender account address (2).

// TODO: in the future, can tie these to the TransactionAuthenticator enum directly with https://github.com/rust-lang/rust/issues/60553
#[derive(Debug)]
#[repr(u8)]
pub enum Scheme {
    Ed25519 = 0,
    MultiEd25519 = 1,
    // ... add more schemes here
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionAuthenticator {
    /// Single signature
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },
    /// K-of-N multisignature
    MultiEd25519 {
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    },
    // ... add more schemes here
}

impl TransactionAuthenticator {
    /// Unique identifier for the signature scheme
    pub fn scheme(&self) -> Scheme {
        match self {
            Self::Ed25519 { .. } => Scheme::Ed25519,
            Self::MultiEd25519 { .. } => Scheme::MultiEd25519,
        }
    }

    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(public_key: Ed25519PublicKey, signature: Ed25519Signature) -> Self {
        Self::Ed25519 {
            public_key,
            signature,
        }
    }

    /// Create a multisignature ed25519 authenticator
    pub fn multi_ed25519(
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    ) -> Self {
        Self::MultiEd25519 {
            public_key,
            signature,
        }
    }

    /// Return Ok if the authenticator's public key matches its signature, Err otherwise
    pub fn verify_signature(&self, message: &HashValue) -> Result<()> {
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => signature.verify(message, public_key),
            Self::MultiEd25519 {
                public_key,
                signature,
            } => signature.verify(message, public_key),
        }
    }

    /// Return the raw bytes of `self.public_key`
    pub fn public_key_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public_key, .. } => public_key.to_bytes().to_vec(),
            Self::MultiEd25519 { public_key, .. } => public_key.to_bytes().to_vec(),
        }
    }

    /// Return the raw bytes of `self.signature`
    pub fn signature_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { signature, .. } => signature.to_bytes().to_vec(),
            Self::MultiEd25519 { signature, .. } => signature.to_bytes().to_vec(),
        }
    }

    /// Return an authentication key preimage derived from `self`'s public key and scheme id
    pub fn authentication_key_preimage(&self) -> AuthenticationKeyPreimage {
        AuthenticationKeyPreimage::new(self.public_key_bytes(), self.scheme())
    }

    /// Return an authentication key derived from `self`'s public key and scheme id
    pub fn authentication_key(&self) -> AuthenticationKey {
        AuthenticationKey::from_preimage(&self.authentication_key_preimage())
    }
}

/// A struct that represents an account authentication key. An account's address is the last 16
/// bytes of authentication key used to create it
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, CryptoHasher)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AuthenticationKey([u8; AuthenticationKey::LENGTH]);

impl AuthenticationKey {
    /// Create an authentication key from `bytes`
    pub const fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// The number of bytes in an authentication key.
    pub const LENGTH: usize = 32;

    /// Create an authentication key from a preimage by taking its sha3 hash
    pub fn from_preimage(preimage: &AuthenticationKeyPreimage) -> AuthenticationKey {
        AuthenticationKey::new(*HashValue::from_sha3_256(&preimage.0).as_ref())
    }

    /// Create an authentication key from an Ed25519 public key
    pub fn ed25519(public_key: &Ed25519PublicKey) -> AuthenticationKey {
        Self::from_preimage(&AuthenticationKeyPreimage::ed25519(public_key))
    }

    /// Create an authentication key from a MultiEd25519 public key
    pub fn multi_ed25519(public_key: &MultiEd25519PublicKey) -> Self {
        Self::from_preimage(&AuthenticationKeyPreimage::multi_ed25519(public_key))
    }

    /// Return an address derived from the last `AccountAddress::LENGTH` bytes of this
    /// authentication key.
    pub fn derived_address(&self) -> AccountAddress {
        // keep only last 16 bytes
        let mut array = [0u8; AccountAddress::LENGTH];
        array.copy_from_slice(&self.0[Self::LENGTH - AccountAddress::LENGTH..]);
        AccountAddress::new(array)
    }

    /// Return the first AccountAddress::LENGTH bytes of this authentication key
    pub fn prefix(&self) -> [u8; AccountAddress::LENGTH] {
        let mut array = [0u8; AccountAddress::LENGTH];
        array.copy_from_slice(&self.0[..AccountAddress::LENGTH]);
        array
    }

    /// Return an abbreviated representation of this authentication key
    pub fn short_str(&self) -> String {
        hex::encode(&self.0[..4])
    }

    /// Construct a vector from this authentication key
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Create a random authentication key. For testing only
    pub fn random() -> Self {
        let mut rng = OsRng::new().expect("can't access OsRng");
        let buf: [u8; Self::LENGTH] = rng.gen();
        AuthenticationKey::new(buf)
    }
}

/// A value that can be hashed to produce an authentication key
pub struct AuthenticationKeyPreimage(Vec<u8>);

impl AuthenticationKeyPreimage {
    /// Return bytes for (public_key | scheme_id)
    fn new(mut public_key_bytes: Vec<u8>, scheme: Scheme) -> Self {
        public_key_bytes.push(scheme as u8);
        Self(public_key_bytes)
    }

    /// Construct a preimage from an Ed25519 public key
    pub fn ed25519(public_key: &Ed25519PublicKey) -> AuthenticationKeyPreimage {
        Self::new(public_key.to_bytes().to_vec(), Scheme::Ed25519)
    }

    /// Construct a preimage from a MultiEd25519 public key
    pub fn multi_ed25519(public_key: &MultiEd25519PublicKey) -> AuthenticationKeyPreimage {
        Self::new(public_key.to_bytes(), Scheme::MultiEd25519)
    }

    /// Construct a slice from this authentication key
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Construct a vector from this authentication key
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Display for TransactionAuthenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TransactionAuthenticator[scheme id: {:?}, public key: {}, signature: {}]",
            self.scheme(),
            hex::encode(&self.public_key_bytes()),
            hex::encode(&self.signature_bytes())
        )
    }
}

impl TryFrom<&[u8]> for AuthenticationKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<AuthenticationKey> {
        ensure!(
            bytes.len() == Self::LENGTH,
            "The authentication key {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; Self::LENGTH];
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
