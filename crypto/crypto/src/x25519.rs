// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An abstraction of x25519 elliptic curve keys required for
//! [Diffie-Hellman key exchange](https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange)
//! in the Diem project.
//! Ideally, only `x25519::PrivateKey` and `x25519::PublicKey` should be used throughout the
//! codebase, until the bytes are actually used in cryptographic operations.
//!
//! # Examples
//!
//! ```
//! use diem_crypto::{x25519, Uniform, test_utils::TEST_SEED};
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! // Derive an X25519 private key for testing.
//! let mut rng: StdRng = SeedableRng::from_seed(TEST_SEED);
//! let private_key = x25519::PrivateKey::generate(&mut rng);
//! let public_key = private_key.public_key();
//!
//! // Deserialize an hexadecimal private or public key
//! use diem_crypto::traits::ValidCryptoMaterialStringExt;
//! # fn main() -> Result<(), diem_crypto::traits::CryptoMaterialError> {
//! let private_key = "404acc8ec6a0f18df7359a6ee7823f19dd95616b10fed8bdb0de030e891b945a";
//! let private_key = x25519::PrivateKey::from_encoded_string(&private_key)?;
//! let public_key = "080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120";
//! let public_key = x25519::PublicKey::from_encoded_string(&public_key)?;
//! # Ok(())
//! # }
//! ```
//!

use crate::{
    traits::{self, CryptoMaterialError, ValidCryptoMaterial, ValidCryptoMaterialStringExt},
    x25519,
};
use diem_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use rand::{CryptoRng, RngCore};
use std::convert::{TryFrom, TryInto};

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;

//
// Underlying Implementation
// =========================
//
// We re-export the dalek-x25519 library,
// This makes it easier to uniformalize build dalek-x25519 in diem-core.
//

pub use x25519_dalek;

//
// Main types and constants
// ========================
//

/// Size of a X25519 private key
pub const PRIVATE_KEY_SIZE: usize = 32;

/// Size of a X25519 public key
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Size of a X25519 shared secret
pub const SHARED_SECRET_SIZE: usize = 32;

/// This type should be used to deserialize a received private key
#[derive(DeserializeKey, SilentDisplay, SilentDebug, SerializeKey)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
pub struct PrivateKey(x25519_dalek::StaticSecret);

/// This type should be used to deserialize a received public key
#[derive(
    Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, SerializeKey, DeserializeKey,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct PublicKey([u8; PUBLIC_KEY_SIZE]);

//
// Handy implementations
// =====================
//

impl PrivateKey {
    /// Obtain the public key part of a private key
    pub fn public_key(&self) -> PublicKey {
        let public_key: x25519_dalek::PublicKey = (&self.0).into();
        PublicKey(public_key.as_bytes().to_owned())
    }

    /// To perform a key exchange with another public key
    pub fn diffie_hellman(&self, remote_public_key: &PublicKey) -> [u8; SHARED_SECRET_SIZE] {
        let remote_public_key = x25519_dalek::PublicKey::from(remote_public_key.0);
        let shared_secret = self.0.diffie_hellman(&remote_public_key);
        shared_secret.as_bytes().to_owned()
    }

    /// Deserialize an X25119 PrivateKey given the sha512 pre-image of a hash
    /// whose least significant half is a canonical X25519 scalar, following
    /// the XEdDSA approach.
    ///
    /// This will FAIL if the passed-in byte representation converts to a
    /// non-canonical scalar in the X25519 sense (and thus cannot correspond to
    /// a X25519 valid key without bit-mangling).
    ///
    /// This is meant to compensate for the poor key storage capabilities of some
    /// key management solutions, and NOT to promote double usage of keys under
    /// several schemes, which would lead to BAD vulnerabilities.
    pub fn from_ed25519_private_bytes(private_slice: &[u8]) -> Result<Self, CryptoMaterialError> {
        let ed25519_secretkey = ed25519_dalek::SecretKey::from_bytes(private_slice)
            .map_err(|_| CryptoMaterialError::DeserializationError)?;
        let expanded_key = ed25519_dalek::ExpandedSecretKey::from(&ed25519_secretkey);

        let mut expanded_keypart = [0u8; 32];
        expanded_keypart.copy_from_slice(&expanded_key.to_bytes()[..32]);
        let potential_x25519 = x25519::PrivateKey::from(expanded_keypart);

        // This checks for x25519 clamping & reduction, which is an RFC requirement
        if potential_x25519.to_bytes()[..] != expanded_key.to_bytes()[..32] {
            Err(CryptoMaterialError::DeserializationError)
        } else {
            Ok(potential_x25519)
        }
    }
}

impl PublicKey {
    /// Obtain a slice reference to the underlying bytearray
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Deserialize an X25119 PublicKey from its representation as an
    /// Ed25519PublicKey, following the XEdDSA approach. This is meant to
    /// compensate for the poor key storage capabilities of key management
    /// solutions, and NOT to promote double usage of keys under several
    /// schemes, which would lead to BAD vulnerabilities.
    pub fn from_ed25519_public_bytes(ed25519_bytes: &[u8]) -> Result<Self, CryptoMaterialError> {
        if ed25519_bytes.len() != 32 {
            return Err(CryptoMaterialError::DeserializationError);
        }
        let ed_point = curve25519_dalek::edwards::CompressedEdwardsY::from_slice(ed25519_bytes)
            .decompress()
            .ok_or(CryptoMaterialError::DeserializationError)?;

        Ok(x25519::PublicKey::from(ed_point.to_montgomery().to_bytes()))
    }
}

//
// Traits implementations
// ======================
//

// private key part

impl std::convert::From<[u8; PRIVATE_KEY_SIZE]> for PrivateKey {
    fn from(private_key_bytes: [u8; PRIVATE_KEY_SIZE]) -> Self {
        Self(x25519_dalek::StaticSecret::from(private_key_bytes))
    }
}

impl std::convert::TryFrom<&[u8]> for PrivateKey {
    type Error = traits::CryptoMaterialError;

    fn try_from(private_key_bytes: &[u8]) -> Result<Self, Self::Error> {
        let private_key_bytes: [u8; PRIVATE_KEY_SIZE] = private_key_bytes
            .try_into()
            .map_err(|_| traits::CryptoMaterialError::DeserializationError)?;
        Ok(Self(x25519_dalek::StaticSecret::from(private_key_bytes)))
    }
}

impl traits::PrivateKey for PrivateKey {
    type PublicKeyMaterial = PublicKey;
}

impl traits::Uniform for PrivateKey {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        Self(x25519_dalek::StaticSecret::new(rng))
    }
}

// TODO: should this be gated under test flag? (mimoo)
impl traits::ValidCryptoMaterial for PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl proptest::arbitrary::Arbitrary for PrivateKey {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        use proptest::strategy::Strategy as _;
        proptest::arbitrary::any::<[u8; 32]>()
            .prop_map(PrivateKey::from)
            .boxed()
    }
}

// public key part

impl From<&PrivateKey> for PublicKey {
    fn from(private_key: &PrivateKey) -> Self {
        private_key.public_key()
    }
}

impl std::convert::From<[u8; PUBLIC_KEY_SIZE]> for PublicKey {
    fn from(public_key_bytes: [u8; PUBLIC_KEY_SIZE]) -> Self {
        Self(public_key_bytes)
    }
}

impl std::convert::TryFrom<&[u8]> for PublicKey {
    type Error = traits::CryptoMaterialError;

    fn try_from(public_key_bytes: &[u8]) -> Result<Self, Self::Error> {
        let public_key_bytes: [u8; PUBLIC_KEY_SIZE] = public_key_bytes
            .try_into()
            .map_err(|_| traits::CryptoMaterialError::WrongLengthError)?;
        Ok(Self(public_key_bytes))
    }
}

impl traits::PublicKey for PublicKey {
    type PrivateKeyMaterial = PrivateKey;
}

impl traits::ValidCryptoMaterial for PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x25519::PublicKey({})", self)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
use crate::test_utils::{self, KeyPair};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;

/// Produces a uniformly random ed25519 keypair from a seed
#[cfg(any(test, feature = "fuzzing"))]
pub fn keypair_strategy() -> impl Strategy<Value = KeyPair<PrivateKey, PublicKey>> {
    test_utils::uniform_keypair_strategy::<PrivateKey, PublicKey>()
}
