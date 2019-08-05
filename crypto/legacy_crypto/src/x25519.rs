// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An implementation of x25519 elliptic curve key pairs required for [Diffie-Hellman key exchange](https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange)
//! in the Libra project.
//!
//! This is an API for [Elliptic Curves for Security - RFC 7748](https://tools.ietf.org/html/rfc7748)
//! and it only deals with long-term key generation and handling.
//!
//! Warning: This API will soon be updated in the [`nextgen`] module.
//!
//! # Examples
//!
//! ```
//! use crypto::x25519::{
//!     derive_keypair_from_seed, generate_keypair, generate_keypair_from_rng,
//!     generate_keypair_hybrid,
//! };
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! // Derive an X25519 from seed using the extract-then-expand HKDF method from RFC 5869.
//! let salt = &b"some salt"[..];
//! // In production, ensure seed has at least 256 bits of entropy.
//! let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
//! let info = &b"some app info"[..];
//!
//! let (private_key1, public_key1) = derive_keypair_from_seed(Some(salt), &seed, Some(info));
//! let (private_key2, public_key2) = derive_keypair_from_seed(Some(salt), &seed, Some(info));
//! assert_eq!(public_key1, public_key2);
//!
//! // Generate a random X25519 key pair.
//! let (private_key, public_key) = generate_keypair();
//!
//! // Generate an X25519 key pair from an RNG (in this example a SeedableRng).
//! let seed = [1u8; 32];
//! let mut rng: StdRng = SeedableRng::from_seed(seed);
//! let (private_key, public_key) = generate_keypair_from_rng(&mut rng);
//!
//! // Generate an X25519 key pair from an RNG and a user-provided seed.
//! let salt = &b"some salt"[..];
//! // In production, ensure seed has at least 256 bits of entropy.
//! let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
//! let info = &b"some app info"[..];
//! let (private_key1, public_key1) = generate_keypair_hybrid(Some(salt), &seed, Some(info));
//! let (private_key2, public_key2) = generate_keypair_hybrid(Some(salt), &seed, Some(info));
//! assert_ne!(public_key1, public_key2);
//! ```
use crate::{hkdf::Hkdf, utils::*};
use core::fmt;
use crypto_derive::{SilentDebug, SilentDisplay};
use failure::prelude::*;
use proptest::{
    arbitrary::any,
    prelude::{Arbitrary, BoxedStrategy},
    strategy::*,
};
use rand::{
    rngs::{EntropyRng, StdRng},
    CryptoRng, RngCore, SeedableRng,
};
use serde::{de, export, ser};
use sha2::Sha256;
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};
use x25519_dalek;

/// An x25519 private key.
#[derive(SilentDisplay, SilentDebug)]
pub struct X25519PrivateKey {
    value: x25519_dalek::StaticSecret,
}

/// An x25519 public key.
#[derive(Copy, Clone)]
pub struct X25519PublicKey {
    value: x25519_dalek::PublicKey,
}

#[deprecated(
    since = "1.0.0",
    note = "This will be superseded by the new cryptography API"
)]
impl Clone for X25519PrivateKey {
    fn clone(&self) -> Self {
        let bytes = self.to_bytes();
        X25519PrivateKey {
            value: x25519_dalek::StaticSecret::from(bytes),
        }
    }
}

impl Deref for X25519PrivateKey {
    type Target = x25519_dalek::StaticSecret;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl X25519PrivateKey {
    /// Length of the private key in bytes.
    pub const LENGTH: usize = 32;
}

impl X25519PublicKey {
    /// Length of the public key in bytes.
    pub const LENGTH: usize = 32;

    /// Obtain a public key from a slice.
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        assert_eq!(
            data.len(),
            X25519PublicKey::LENGTH,
            "X25519 Public key wrong length error; expected {} but received {}",
            X25519PublicKey::LENGTH,
            data.len()
        );
        let mut fixed_size_data: [u8; X25519PublicKey::LENGTH] = Default::default();
        fixed_size_data.copy_from_slice(data);
        let key = x25519_dalek::PublicKey::from(fixed_size_data);
        Ok(X25519PublicKey { value: key })
    }

    /// Convert a public key into a slice.
    pub fn to_slice(&self) -> [u8; Self::LENGTH] {
        *self.value.as_bytes()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.to_slice()))
    }
}

impl PartialEq<X25519PublicKey> for X25519PublicKey {
    fn eq(&self, other: &X25519PublicKey) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for X25519PublicKey {}

impl Display for X25519PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        X25519PublicKey::fmt(self, f)
    }
}

impl Debug for X25519PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        X25519PublicKey::fmt(self, f)
    }
}

impl Deref for X25519PublicKey {
    type Target = x25519_dalek::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl ser::Serialize for X25519PrivateKey {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl ser::Serialize for X25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl From<&X25519PrivateKey> for X25519PublicKey {
    fn from(private_key: &X25519PrivateKey) -> Self {
        let public_key = (&private_key.value).into();
        Self { value: public_key }
    }
}

struct X25519PrivateKeyVisitor;
struct X25519PublicKeyVisitor;

impl<'de> de::Visitor<'de> for X25519PrivateKeyVisitor {
    type Value = X25519PrivateKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("X25519_dalek private key in bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<X25519PrivateKey, E>
    where
        E: de::Error,
    {
        let mut fixed_size_data: [u8; X25519PrivateKey::LENGTH] = Default::default();
        fixed_size_data.copy_from_slice(value);
        let key = x25519_dalek::StaticSecret::from(fixed_size_data);
        Ok(X25519PrivateKey { value: key })
    }
}

impl<'de> de::Visitor<'de> for X25519PublicKeyVisitor {
    type Value = X25519PublicKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("X25519_dalek public key in bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<X25519PublicKey, E>
    where
        E: de::Error,
    {
        let mut fixed_size_data: [u8; X25519PublicKey::LENGTH] = Default::default();
        fixed_size_data.copy_from_slice(value);
        let key = x25519_dalek::PublicKey::from(fixed_size_data);
        Ok(X25519PublicKey { value: key })
    }
}

impl<'de> de::Deserialize<'de> for X25519PrivateKey {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(X25519PrivateKeyVisitor {})
    }
}

impl<'de> de::Deserialize<'de> for X25519PublicKey {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(X25519PublicKeyVisitor {})
    }
}

impl Arbitrary for X25519PublicKey {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        public_key_strategy().boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

fn public_key_strategy() -> impl Strategy<Value = X25519PublicKey> {
    any::<[u8; X25519PublicKey::LENGTH]>()
        .prop_map(|seed| {
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            let (_, public_key) = generate_keypair_from_rng(&mut rng);
            public_key
        })
        .no_shrink()
}

/// Generates a consistent keypair `(X25519PrivateKey, X25519PublicKey)` for unit tests.
pub fn generate_keypair_for_testing<R>(rng: &mut R) -> (X25519PrivateKey, X25519PublicKey)
where
    R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
{
    generate_keypair_from_rng(rng)
}

/// Generates a random key-pair `(X25519PrivateKey, X25519PublicKey)`.
pub fn generate_keypair() -> (X25519PrivateKey, X25519PublicKey) {
    let mut rng = EntropyRng::new();
    generate_keypair_from_rng(&mut rng)
}

/// Derives a keypair `(X25519PrivateKey, X25519PublicKey)` from
/// a) salt (optional) - denoted as 'salt' in RFC 5869
/// b) seed - denoted as 'IKM' in RFC 5869
/// c) application info (optional) - denoted as 'info' in RFC 5869
///
/// using the HKDF key derivation protocol, as defined in RFC 5869.
/// This implementation uses the full extract-then-expand HKDF steps
/// based on the SHA-256 hash function.
///
/// **Warning**: This function will soon be updated to return a KeyPair struct.
pub fn derive_keypair_from_seed(
    salt: Option<&[u8]>,
    seed: &[u8],
    app_info: Option<&[u8]>,
) -> (X25519PrivateKey, X25519PublicKey) {
    let derived_bytes =
        Hkdf::<Sha256>::extract_then_expand(salt, seed, app_info, X25519PrivateKey::LENGTH);
    let mut key_bytes = [0u8; X25519PrivateKey::LENGTH];
    key_bytes.copy_from_slice(derived_bytes.unwrap().as_slice());

    let secret: x25519_dalek::StaticSecret = x25519_dalek::StaticSecret::from(key_bytes);
    let public: x25519_dalek::PublicKey = (&secret).into();
    (
        X25519PrivateKey { value: secret },
        X25519PublicKey { value: public },
    )
}

/// Generates a keypair `(X25519PrivateKey, X25519PublicKey)` based on an RNG.
pub fn generate_keypair_from_rng<R>(rng: &mut R) -> (X25519PrivateKey, X25519PublicKey)
where
    R: RngCore + CryptoRng,
{
    let secret: x25519_dalek::StaticSecret = x25519_dalek::StaticSecret::new(rng);
    let public: x25519_dalek::PublicKey = (&secret).into();
    (
        X25519PrivateKey { value: secret },
        X25519PublicKey { value: public },
    )
}

/// Generates a random keypair `(X25519PrivateKey, X25519PublicKey)` and returns string
/// representations tuple:
/// 1. human readable public key.
/// 2. hex encoded serialized public key.
/// 3. hex encoded serialized private key.
pub fn generate_and_encode_keypair() -> (String, String, String) {
    let (private_key, public_key) = generate_keypair();
    let pub_key_human = hex::encode(public_key.to_slice());
    let public_key_serialized_str = encode_to_string(&public_key);
    let private_key_serialized_str = encode_to_string(&private_key);
    (
        pub_key_human,
        public_key_serialized_str,
        private_key_serialized_str,
    )
}

/// Generates a random keypair `(PrivateKey, PublicKey)` by combining the output of `EntropyRng`
/// with a user-provided seed. This concatenated seed is used as the seed to HKDF (RFC 5869).
///
/// Similarly to `derive_keypair_from_seed` the user provides the following inputs:
/// a) salt (optional) - denoted as 'salt' in RFC 5869
/// b) seed - denoted as 'IKM' in RFC 5869
/// c) application info (optional) - denoted as 'info' in RFC 5869
///
/// Note that this method is not deterministic, but the (random + static seed) key
/// generation makes it safer against low entropy pools and weak RNGs.
///
/// **Warning**: This function will soon be updated to return a [`KeyPair`] struct.
pub fn generate_keypair_hybrid(
    salt: Option<&[u8]>,
    seed: &[u8],
    app_info: Option<&[u8]>,
) -> (X25519PrivateKey, X25519PublicKey) {
    let mut rng = EntropyRng::new();
    let mut seed_from_rng = [0u8; ed25519_dalek::SECRET_KEY_LENGTH];
    rng.fill_bytes(&mut seed_from_rng);

    let mut final_seed = seed.to_vec();
    final_seed.extend_from_slice(&seed_from_rng);

    derive_keypair_from_seed(salt, &final_seed, app_info)
}
