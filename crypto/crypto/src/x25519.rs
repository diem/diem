// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An implementation of x25519 elliptic curve key pairs required for
//! [Diffie-Hellman key
//! exchange](https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange)
//! in the Libra project.
//!
//! This is an API for [Elliptic Curves for Security - RFC
//! 7748](https://tools.ietf.org/html/rfc7748) and which deals with
//! long-term key generation and handling (`X25519StaticPrivateKey`,
//! `X25519StaticPublicKey`) as well as short-term keys (`X25519EphemeralPrivateKey`,
//! `X25519PublicKey`).
//!
//! The default type for a Diffie-Hellman secret is an ephemeral
//! one, forming a `PrivateKey`-`PublicKey` pair with `X25519Publickey`,
//! and is not serializable, since the use of fresh DH secrets is
//! recommended for various reasons including PFS.
//!
//! We also provide a "static" implementation `X25519StaticPrivateKey`,
//! which supports serialization, forming a `PrivateKey`-`PublicKey` pair
//! with `X25519StaticPublickey`. This later type is precisely a
//! [newtype](https://doc.rust-lang.org/1.5.0/style/features/types/newtype.html)
//! wrapper around `X25519PublicKey`, to which it coerces through `Deref`.
//!
//! # Examples
//!
//! ```
//! use libra_crypto::x25519::*;
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! // Derive an X25519 static key pair from seed using the extract-then-expand HKDF method from RFC 5869.
//! let salt = &b"some salt"[..];
//! // In production, ensure seed has at least 256 bits of entropy.
//! let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
//! let info = &b"some app info"[..];
//!
//! let (private_key1, public_key1) = X25519StaticPrivateKey::derive_keypair_from_seed(Some(salt), &seed, Some(info));
//! let (private_key2, public_key2) = X25519StaticPrivateKey::derive_keypair_from_seed(Some(salt), &seed, Some(info));
//! assert_eq!(public_key1, public_key2);
//!
//! // Generate a random X25519 ephemeral key pair from an RNG (in this example a StdRng)
//! use libra_crypto::Uniform;
//! let seed = [1u8; 32];
//! let mut rng: StdRng = SeedableRng::from_seed(seed);
//! let private_key = X25519StaticPrivateKey::generate(&mut rng);
//! let public_key: X25519StaticPublicKey = (&private_key).into();
//!
//! // Generate an X25519 key pair from an RNG and a user-provided seed.
//! let salt = &b"some salt"[..];
//! // In production, ensure seed has at least 256 bits of entropy.
//! let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
//! let info = &b"some app info"[..];
//! let (private_key1, public_key1) = X25519StaticPrivateKey::generate_keypair_hybrid(Some(salt), &seed, Some(info));
//! let (private_key2, public_key2) = X25519StaticPrivateKey::generate_keypair_hybrid(Some(salt), &seed, Some(info));
//! assert_ne!(public_key1, public_key2);
//! ```

use crate::{hkdf::Hkdf, traits::*};
use libra_crypto_derive::{Deref, DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use rand::{rngs::EntropyRng, RngCore};
use sha2::Sha256;
use std::{convert::TryFrom, ops::Deref};

/// TODO: move traits to the right file (possibly traits.rs)

/// Key interfaces for Diffie-Hellman key exchange protocol build on top
/// of the key APIs in traits.rs

/// x25519 implementation

/// The length of the DHPublicKey
pub const X25519_PUBLIC_KEY_LENGTH: usize = 32;
/// The length of the DHPrivateKey
pub const X25519_PRIVATE_KEY_LENGTH: usize = 32;

/// An x25519 ephemeral private (secret) key
#[derive(SilentDisplay, SilentDebug)]
pub struct X25519EphemeralPrivateKey(x25519_dalek::EphemeralSecret);

/// An x25519 static private (secret) key
#[derive(Clone, DeserializeKey, SilentDisplay, SilentDebug, SerializeKey)]
pub struct X25519StaticPrivateKey(x25519_dalek::StaticSecret);

/// An x25519 public key
#[derive(Clone, Deref)]
pub struct X25519PublicKey(x25519_dalek::PublicKey);

/// An x25519 public key to match the X25519Static key type, which
/// dereferences to an X25519PublicKey
#[derive(Clone, DeserializeKey, Eq, Hash, PartialEq, SerializeKey)]
pub struct X25519StaticPublicKey(X25519PublicKey);

/// An x25519 shared key
#[derive(SilentDisplay, SilentDebug)]
pub struct X25519SharedKey(x25519_dalek::SharedSecret);

/////////////////////////
// X25519EphemeralPrivateKey Traits //
/////////////////////////

impl Uniform for X25519EphemeralPrivateKey {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        X25519EphemeralPrivateKey(x25519_dalek::EphemeralSecret::new(rng))
    }
}

impl PrivateKey for X25519EphemeralPrivateKey {
    type PublicKeyMaterial = X25519PublicKey;
}

impl ExchangeKey for X25519EphemeralPrivateKey {
    type DHPublicKeyMaterial = X25519PublicKey;
    type DHSharedKeyMaterial = X25519SharedKey;

    // Diffie-Hellman exchange
    fn dh(self, their_public: &X25519PublicKey) -> X25519SharedKey {
        let shared_secret = self.0.diffie_hellman(&their_public.0);
        X25519SharedKey(shared_secret)
    }
}

//////////////////////
// X25519StaticPrivateKey Traits //
//////////////////////

impl X25519StaticPrivateKey {
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
    ) -> (X25519StaticPrivateKey, X25519StaticPublicKey) {
        let derived_bytes =
            Hkdf::<Sha256>::extract_then_expand(salt, seed, app_info, X25519_PRIVATE_KEY_LENGTH);
        let mut key_bytes = [0u8; X25519_PRIVATE_KEY_LENGTH];
        key_bytes.copy_from_slice(derived_bytes.unwrap().as_slice());

        let secret: x25519_dalek::StaticSecret = x25519_dalek::StaticSecret::from(key_bytes);
        let public: x25519_dalek::PublicKey = (&secret).into();
        (
            X25519StaticPrivateKey(secret),
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
    ) -> (X25519StaticPrivateKey, X25519StaticPublicKey) {
        let mut rng = EntropyRng::new();
        let mut seed_from_rng = [0u8; X25519_PRIVATE_KEY_LENGTH];
        rng.fill_bytes(&mut seed_from_rng);

        let mut final_seed = seed.to_vec();
        final_seed.extend_from_slice(&seed_from_rng);

        X25519StaticPrivateKey::derive_keypair_from_seed(salt, &final_seed, app_info)
    }
}

impl Uniform for X25519StaticPrivateKey {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        X25519StaticPrivateKey(x25519_dalek::StaticSecret::new(rng))
    }
}

impl PrivateKey for X25519StaticPrivateKey {
    type PublicKeyMaterial = X25519StaticPublicKey;
}

impl ExchangeKey for X25519StaticPrivateKey {
    type DHPublicKeyMaterial = X25519StaticPublicKey;
    type DHSharedKeyMaterial = X25519SharedKey;

    // Diffie-Hellman exchange
    fn dh(self, their_public: &X25519StaticPublicKey) -> X25519SharedKey {
        let shared_secret = self.0.diffie_hellman(&(their_public.deref()).0);
        X25519SharedKey(shared_secret)
    }
}

impl PartialEq for X25519StaticPrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl TryFrom<&[u8]> for X25519StaticPrivateKey {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> Result<X25519StaticPrivateKey, CryptoMaterialError> {
        if bytes.len() != X25519_PRIVATE_KEY_LENGTH {
            return Err(CryptoMaterialError::DeserializationError);
        }
        let mut bits = [0u8; X25519_PRIVATE_KEY_LENGTH];
        bits.copy_from_slice(&bytes[..X25519_PRIVATE_KEY_LENGTH]);
        Ok(X25519StaticPrivateKey(x25519_dalek::StaticSecret::from(
            bits,
        )))
    }
}

impl Length for X25519StaticPrivateKey {
    fn length(&self) -> usize {
        X25519_PRIVATE_KEY_LENGTH
    }
}

impl ValidKey for X25519StaticPrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

//////////////////////
// X25519PublicKey Traits //
//////////////////////

impl<'a> From<&'a X25519EphemeralPrivateKey> for X25519PublicKey {
    fn from(ephemeral: &'a X25519EphemeralPrivateKey) -> X25519PublicKey {
        X25519PublicKey(x25519_dalek::PublicKey::from(&ephemeral.0))
    }
}

impl<'a> From<&'a X25519StaticPrivateKey> for X25519StaticPublicKey {
    fn from(ephemeral: &'a X25519StaticPrivateKey) -> X25519StaticPublicKey {
        X25519StaticPublicKey(X25519PublicKey(x25519_dalek::PublicKey::from(&ephemeral.0)))
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
    type PrivateKeyMaterial = X25519EphemeralPrivateKey;
}

impl PublicKey for X25519StaticPublicKey {
    type PrivateKeyMaterial = X25519StaticPrivateKey;
}

impl TryFrom<&[u8]> for X25519StaticPublicKey {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> Result<X25519StaticPublicKey, CryptoMaterialError> {
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

impl Length for X25519StaticPublicKey {
    fn length(&self) -> usize {
        X25519_PUBLIC_KEY_LENGTH
    }
}

impl ValidKey for X25519StaticPublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.deref().0.as_bytes().to_vec()
    }
}

impl std::fmt::Display for X25519PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0.as_bytes()))
    }
}

impl std::fmt::Debug for X25519PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "X25519PublicKey({})", self)
    }
}

impl std::fmt::Display for X25519StaticPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::fmt::Debug for X25519StaticPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "X25519StaticPublicKey({})", self)
    }
}

//////////////////////////
// Compatibility Traits //
//////////////////////////

/// Those transitory traits are meant to help with the progressive
/// migration of the code base to the crypto module and will
/// disappear after.
pub mod compat {
    use crate::traits::*;
    #[cfg(any(test, feature = "fuzzing"))]
    use proptest::strategy::LazyJust;
    #[cfg(any(test, feature = "fuzzing"))]
    use proptest::{prelude::*, strategy::Strategy};

    use crate::x25519::{X25519StaticPrivateKey, X25519StaticPublicKey};
    use rand::{rngs::StdRng, SeedableRng};

    /// Generate an arbitrary key pair, with possible Rng input
    ///
    /// Warning: if you pass in None, this will not return distinct
    /// results every time! Should you want to write non-deterministic
    /// tests, look at libra_config::config_builder::util::get_test_config
    pub fn generate_keypair<'a, T>(opt_rng: T) -> (X25519StaticPrivateKey, X25519StaticPublicKey)
    where
        T: Into<Option<&'a mut StdRng>> + Sized,
    {
        if let Some(rng_mut_ref) = opt_rng.into() {
            <(X25519StaticPrivateKey, X25519StaticPublicKey)>::generate(rng_mut_ref)
        } else {
            let mut rng = StdRng::from_seed(crate::test_utils::TEST_SEED);
            <(X25519StaticPrivateKey, X25519StaticPublicKey)>::generate(&mut rng)
        }
    }

    /// Used to produce keypairs from a seed for testing purposes
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn keypair_strategy(
    ) -> impl Strategy<Value = (X25519StaticPrivateKey, X25519StaticPublicKey)> {
        // The no_shrink is because keypairs should be fixed -- shrinking would cause a different
        // keypair to be generated, which appears to not be very useful.
        any::<[u8; 32]>()
            .prop_map(|seed| {
                let mut rng: StdRng = SeedableRng::from_seed(seed);
                let (private_key, public_key) = generate_keypair(&mut rng);
                (private_key, public_key)
            })
            .no_shrink()
    }

    #[cfg(any(test, feature = "fuzzing"))]
    impl Arbitrary for X25519StaticPublicKey {
        type Parameters = ();
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            LazyJust::new(|| generate_keypair(None).1).boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}
