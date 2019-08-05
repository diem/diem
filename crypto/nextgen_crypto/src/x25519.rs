// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An implementation of x25519 elliptic curve key pairs required for
//! [Diffie-Hellman key
//! exchange](https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange)
//! in the Libra project.
//!
//! This is an API for [Elliptic Curves for Security - RFC
//! 7748](https://tools.ietf.org/html/rfc7748) and which deals with
//! long-term key generation and handling (`X25519StaticExchangeKey`,
//! `X25519StaticPublicKey`) as well as short-term keys (`X25519ExchangeKey`,
//! `X25519PublicKey`).
//!
//! The default type for a Diffie-Hellman secret is an ephemeral
//! one, forming a `PrivateKey`-`PublicKey` pair with `X25519Publickey`,
//! and is not serializable, since the use of fresh DH secrets is
//! recommended for various reasons including PFS.
//!
//! We also provide a "static" implementation `X25519StaticExchangeKey`,
//! which supports serialization, forming a `PrivateKey`-`PublicKey` pair
//! with `X25519StaticPublickey`. This later type is precisely a
//! [newtype](https://doc.rust-lang.org/1.5.0/style/features/types/newtype.html)
//! wrapper around `X25519PublicKey`, to which it coerces through `Deref`.
//!
//! # Examples
//!
//! ```
//! use nextgen_crypto::x25519::*;
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! // Derive an X25519 static key pair from seed using the extract-then-expand HKDF method from RFC 5869.
//! let salt = &b"some salt"[..];
//! // In production, ensure seed has at least 256 bits of entropy.
//! let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
//! let info = &b"some app info"[..];
//!
//! let (private_key1, public_key1) = X25519StaticExchangeKey::derive_keypair_from_seed(Some(salt), &seed, Some(info));
//! let (private_key2, public_key2) = X25519StaticExchangeKey::derive_keypair_from_seed(Some(salt), &seed, Some(info));
//! assert_eq!(public_key1, public_key2);
//!
//! // Generate a random X25519 ephemeral key pair from an RNG (in this example a StdRng)
//! use nextgen_crypto::Uniform;
//! let seed = [1u8; 32];
//! let mut rng: StdRng = SeedableRng::from_seed(seed);
//! let private_key = X25519StaticExchangeKey::generate_for_testing(&mut rng);
//! let public_key: X25519StaticPublicKey = (&private_key).into();
//!
//! // Generate an X25519 key pair from an RNG and a user-provided seed.
//! let salt = &b"some salt"[..];
//! // In production, ensure seed has at least 256 bits of entropy.
//! let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
//! let info = &b"some app info"[..];
//! let (private_key1, public_key1) = X25519StaticExchangeKey::generate_keypair_hybrid(Some(salt), &seed, Some(info));
//! let (private_key2, public_key2) = X25519StaticExchangeKey::generate_keypair_hybrid(Some(salt), &seed, Some(info));
//! assert_ne!(public_key1, public_key2);
//! ```

use crate::traits::*;
use crypto::hkdf::Hkdf;
use crypto_derive::{SilentDebug, SilentDisplay};
use rand::{rngs::EntropyRng, RngCore};
use serde::{de, export, ser};
use sha2::Sha256;
use std::{convert::TryFrom, fmt, ops::Deref};
use x25519_dalek;

/// TODO: move traits to the right file (possibly traits.rs)

/// Key interfaces for Diffie-Hellman key exchange protocol build on top
/// of the key APIs in traits.rs

/// x25519 Implementation

/// The length of the DHPublicKey
pub const X25519_PUBLIC_KEY_LENGTH: usize = 32;
/// The length of the DHPrivateKey
pub const X25519_PRIVATE_KEY_LENGTH: usize = 32;

/// An x25519 ephemeral key
#[derive(SilentDisplay, SilentDebug)]
pub struct X25519ExchangeKey(x25519_dalek::EphemeralSecret);

/// An x25519 static key
#[derive(SilentDisplay, SilentDebug, Clone)]
pub struct X25519StaticExchangeKey(x25519_dalek::StaticSecret);

/// An x25519 public key
#[derive(Clone, Debug)]
pub struct X25519PublicKey(x25519_dalek::PublicKey);

/// An x25519 public key to match the X25519Static key type, which
/// dereferences to an X2519PublicKey
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct X25519StaticPublicKey(X25519PublicKey);

/// An x25519 shared key
#[derive(SilentDisplay, SilentDebug)]
pub struct X25519SharedKey(x25519_dalek::SharedSecret);

/////////////////////////
// ExchangeKey Traits //
/////////////////////////

impl Uniform for X25519ExchangeKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        X25519ExchangeKey(x25519_dalek::EphemeralSecret::new(rng))
    }
}

impl PrivateKey for X25519ExchangeKey {
    type PublicKeyMaterial = X25519PublicKey;
}

impl ExchangeKey for X25519ExchangeKey {
    type DHPublicKeyMaterial = X25519PublicKey;
    type DHSharedKeyMaterial = X25519SharedKey;

    // Diffie-Hellman exchange
    fn dh(self, their_public: &X25519PublicKey) -> X25519SharedKey {
        let shared_secret = self.0.diffie_hellman(&their_public.0);
        X25519SharedKey(shared_secret)
    }
}

//////////////////////
// StaticExchangeKey Traits //
//////////////////////

impl X25519StaticExchangeKey {
    /// Derives a keypair `(X25519PrivateKey, X25519PublicKey)` from
    /// a) salt (optional) - denoted as 'salt' in RFC 5869
    /// b) seed - denoted as 'IKM' in RFC 5869
    /// c) application info (optional) - denoted as 'info' in RFC 5869
    ///
    /// using the HKDF key derivation protocol, as defined in RFC 5869.
    /// This implementation uses the full extract-then-expand HKDF steps
    /// based on the SHA-256 hash function.
    pub fn derive_keypair_from_seed(
        salt: Option<&[u8]>,
        seed: &[u8],
        app_info: Option<&[u8]>,
    ) -> (X25519StaticExchangeKey, X25519StaticPublicKey) {
        let derived_bytes =
            Hkdf::<Sha256>::extract_then_expand(salt, seed, app_info, X25519_PRIVATE_KEY_LENGTH);
        let mut key_bytes = [0u8; X25519_PRIVATE_KEY_LENGTH];
        key_bytes.copy_from_slice(derived_bytes.unwrap().as_slice());

        let secret: x25519_dalek::StaticSecret = x25519_dalek::StaticSecret::from(key_bytes);
        let public: x25519_dalek::PublicKey = (&secret).into();
        (
            X25519StaticExchangeKey(secret),
            X25519StaticPublicKey(X25519PublicKey(public)),
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
    pub fn generate_keypair_hybrid(
        salt: Option<&[u8]>,
        seed: &[u8],
        app_info: Option<&[u8]>,
    ) -> (X25519StaticExchangeKey, X25519StaticPublicKey) {
        let mut rng = EntropyRng::new();
        let mut seed_from_rng = [0u8; X25519_PRIVATE_KEY_LENGTH];
        rng.fill_bytes(&mut seed_from_rng);

        let mut final_seed = seed.to_vec();
        final_seed.extend_from_slice(&seed_from_rng);

        X25519StaticExchangeKey::derive_keypair_from_seed(salt, &final_seed, app_info)
    }
}

impl Uniform for X25519StaticExchangeKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        X25519StaticExchangeKey(x25519_dalek::StaticSecret::new(rng))
    }
}

impl PrivateKey for X25519StaticExchangeKey {
    type PublicKeyMaterial = X25519StaticPublicKey;
}

impl ExchangeKey for X25519StaticExchangeKey {
    type DHPublicKeyMaterial = X25519StaticPublicKey;
    type DHSharedKeyMaterial = X25519SharedKey;

    // Diffie-Hellman exchange
    fn dh(self, their_public: &X25519StaticPublicKey) -> X25519SharedKey {
        let shared_secret = self.0.diffie_hellman(&(their_public.deref()).0);
        X25519SharedKey(shared_secret)
    }
}

impl TryFrom<&[u8]> for X25519StaticExchangeKey {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> std::result::Result<X25519StaticExchangeKey, CryptoMaterialError> {
        if bytes.len() != X25519_PRIVATE_KEY_LENGTH {
            return Err(CryptoMaterialError::DeserializationError);
        }
        let mut bits = [0u8; X25519_PRIVATE_KEY_LENGTH];
        bits.copy_from_slice(&bytes[..X25519_PRIVATE_KEY_LENGTH]);
        Ok(X25519StaticExchangeKey(x25519_dalek::StaticSecret::from(
            bits,
        )))
    }
}

impl ValidKey for X25519StaticExchangeKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////

impl<'a> From<&'a X25519ExchangeKey> for X25519PublicKey {
    fn from(ephemeral: &'a X25519ExchangeKey) -> X25519PublicKey {
        X25519PublicKey(x25519_dalek::PublicKey::from(&ephemeral.0))
    }
}

impl<'a> From<&'a X25519StaticExchangeKey> for X25519StaticPublicKey {
    fn from(ephemeral: &'a X25519StaticExchangeKey) -> X25519StaticPublicKey {
        X25519StaticPublicKey(X25519PublicKey(x25519_dalek::PublicKey::from(&ephemeral.0)))
    }
}

impl Deref for X25519StaticPublicKey {
    type Target = X25519PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::hash::Hash for X25519PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.0.as_bytes();
        state.write(encoded_pubkey);
    }
}

impl PartialEq for X25519PublicKey {
    fn eq(&self, other: &X25519PublicKey) -> bool {
        *self.0.as_bytes() == *other.0.as_bytes()
    }
}

impl Eq for X25519PublicKey {}

impl PublicKey for X25519PublicKey {
    type PrivateKeyMaterial = X25519ExchangeKey;

    fn length() -> usize {
        X25519_PUBLIC_KEY_LENGTH
    }
}

impl PublicKey for X25519StaticPublicKey {
    type PrivateKeyMaterial = X25519StaticExchangeKey;

    fn length() -> usize {
        X25519_PUBLIC_KEY_LENGTH
    }
}

impl TryFrom<&[u8]> for X25519StaticPublicKey {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> std::result::Result<X25519StaticPublicKey, CryptoMaterialError> {
        if bytes.len() != X25519_PUBLIC_KEY_LENGTH {
            return Err(CryptoMaterialError::DeserializationError);
        }
        let mut bits = [0u8; X25519_PRIVATE_KEY_LENGTH];
        bits.copy_from_slice(&bytes[..X25519_PRIVATE_KEY_LENGTH]);
        Ok(X25519StaticPublicKey(X25519PublicKey(
            x25519_dalek::PublicKey::from(bits),
        )))
    }
}

impl ValidKey for X25519StaticPublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.deref().0.as_bytes().to_vec()
    }
}

//////////////////////
// SharedKey Traits //
//////////////////////

//////////////////////////////
// Compact Serialization    //
//////////////////////////////

impl ser::Serialize for X25519StaticExchangeKey {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl ser::Serialize for X25519StaticPublicKey {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

struct X25519StaticExchangeKeyVisitor;

struct X25519StaticPublicKeyVisitor;

impl<'de> de::Visitor<'de> for X25519StaticExchangeKeyVisitor {
    type Value = X25519StaticExchangeKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("x25519_dalek static key in bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<X25519StaticExchangeKey, E>
    where
        E: de::Error,
    {
        match X25519StaticExchangeKey::try_from(value) {
            Ok(key) => Ok(key),
            Err(error) => Err(E::custom(error)),
        }
    }
}

impl<'de> de::Visitor<'de> for X25519StaticPublicKeyVisitor {
    type Value = X25519StaticPublicKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("x25519_dalek public key in bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<X25519StaticPublicKey, E>
    where
        E: de::Error,
    {
        match X25519StaticPublicKey::try_from(value) {
            Ok(key) => Ok(key),
            Err(error) => Err(E::custom(error)),
        }
    }
}

impl<'de> de::Deserialize<'de> for X25519StaticExchangeKey {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(X25519StaticExchangeKeyVisitor {})
    }
}

impl<'de> de::Deserialize<'de> for X25519StaticPublicKey {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(X25519StaticPublicKeyVisitor {})
    }
}
