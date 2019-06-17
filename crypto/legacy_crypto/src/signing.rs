// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An implementation of the associated functions for handling
//! [cryptographic signatures](https://en.wikipedia.org/wiki/Digital_signature)
//! for the Libra project.
//!
//! This is an API for [Pure Ed25519 EdDSA signatures](https://tools.ietf.org/html/rfc8032).
//!
//! Warning: This API will soon be updated in the [`nextgen`] module.
//!
//! # Example
//!
//! Note that the signing and verifying functions take an input message of type [`HashValue`].
//! ```
//! use crypto::{hash::*, signing::*};
//!
//! let mut hasher = TestOnlyHasher::default();
//! hasher.write("Test message".as_bytes());
//! let hashed_message = hasher.finish();
//!
//! let (private_key, public_key) = generate_keypair();
//! let signature = sign_message(hashed_message, &private_key).unwrap();
//! assert!(verify_message(hashed_message, &signature, &public_key).is_ok());
//! ```

use crate::{hash::HashValue, hkdf::Hkdf, utils::*};
use bincode::{deserialize, serialize};
use curve25519_dalek::scalar::Scalar;
use ed25519_dalek;
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
use serde::{de, export, ser, Deserialize, Serialize};
use sha2::Sha256;
use std::{clone::Clone, fmt, hash::Hash};

/// An ed25519 private key.
pub struct PrivateKey {
    value: ed25519_dalek::SecretKey,
}

/// An ed25519 public key.
#[derive(Copy, Clone, Default)]
pub struct PublicKey {
    value: ed25519_dalek::PublicKey,
}

/// An ed25519 signature.
#[derive(Copy, Clone)]
pub struct Signature {
    value: ed25519_dalek::Signature,
}

/// A public and private key pair.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct KeyPair {
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl KeyPair {
    /// Produces a new keypair from a private key.
    pub fn new(private_key: PrivateKey) -> Self {
        let public: ed25519_dalek::PublicKey = (&private_key.value).into();
        let public_key = PublicKey { value: public };
        Self {
            private_key,
            public_key,
        }
    }

    /// Returns the public key.
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    /// Returns the private key.
    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }
}

/// Constructs a signature for `message` using `private_key`.
pub fn sign_message(message: HashValue, private_key: &PrivateKey) -> Result<Signature> {
    let public_key: ed25519_dalek::PublicKey = (&private_key.value).into();
    let expanded_secret_key: ed25519_dalek::ExpandedSecretKey =
        ed25519_dalek::ExpandedSecretKey::from(&private_key.value);
    Ok(Signature {
        value: expanded_secret_key.sign(message.as_ref(), &public_key),
    })
}

/// Checks that `signature` is valid for `message` using `public_key`.
pub fn verify_message(
    message: HashValue,
    signature: &Signature,
    public_key: &PublicKey,
) -> Result<()> {
    signature.is_valid()?;
    public_key
        .value
        .verify(message.as_ref(), &signature.value)?;
    Ok(())
}

/// Generates a well-known keypair `(PrivateKey, PublicKey)` for special use
/// in the genesis block.
///
/// **Warning**: This function will soon be updated to return a [`KeyPair`] struct
pub fn generate_genesis_keypair() -> (PrivateKey, PublicKey) {
    let mut buf = [0u8; ed25519_dalek::SECRET_KEY_LENGTH];
    buf[ed25519_dalek::SECRET_KEY_LENGTH - 1] = 1;
    let secret_key: ed25519_dalek::SecretKey = ed25519_dalek::SecretKey::from_bytes(&buf).unwrap();
    let public: ed25519_dalek::PublicKey = (&secret_key).into();
    (
        PrivateKey { value: secret_key },
        PublicKey { value: public },
    )
}

/// Generates a random keypair `(PrivateKey, PublicKey)`.
///
/// **Warning**: This function will soon be updated to return a [`KeyPair`] struct.
pub fn generate_keypair() -> (PrivateKey, PublicKey) {
    let mut rng = EntropyRng::new();
    generate_keypair_from_rng(&mut rng)
}

/// Derives a keypair `(PrivateKey, PublicKey)` from
/// a) salt (optional) - denoted as 'salt' in RFC 5869
/// b) seed - denoted as 'IKM' in RFC 5869
/// c) application info (optional) - denoted as 'info' in RFC 5869
///
/// using the HKDF key derivation protocol, as defined in RFC 5869.
/// This implementation uses the full extract-then-expand HKDF steps
/// based on the SHA-256 hash function.
///
/// **Warning**: This function will soon be updated to return a [`KeyPair`] struct.
pub fn derive_keypair_from_seed(
    salt: Option<&[u8]>,
    seed: &[u8],
    app_info: Option<&[u8]>,
) -> (PrivateKey, PublicKey) {
    let derived_bytes =
        Hkdf::<Sha256>::extract_then_expand(salt, seed, app_info, ed25519_dalek::SECRET_KEY_LENGTH);

    let secret = ed25519_dalek::SecretKey::from_bytes(&derived_bytes.unwrap()).unwrap();
    let public: ed25519_dalek::PublicKey = (&secret).into();
    (PrivateKey { value: secret }, PublicKey { value: public })
}

/// Generates a random keypair `(PrivateKey, PublicKey)` and returns a tuple of string
/// representations:
/// 1. human readable public key
/// 2. hex encoded serialized public key
/// 3. hex encoded serialized private key
pub fn generate_and_encode_keypair() -> (String, String, String) {
    let (private_key, public_key) = generate_keypair();
    let pub_key_human = hex::encode(public_key.value.to_bytes());
    let public_key_serialized_str = encode_to_string(&public_key);
    let private_key_serialized_str = encode_to_string(&private_key);
    (
        pub_key_human,
        public_key_serialized_str,
        private_key_serialized_str,
    )
}

/// Generates consistent keypair `(PrivateKey, PublicKey)` for unit tests.
///
/// **Warning**: This function will soon be updated to return a [`KeyPair`] struct.
pub fn generate_keypair_for_testing<R>(rng: &mut R) -> (PrivateKey, PublicKey)
where
    R: SeedableRng + RngCore + CryptoRng,
{
    generate_keypair_from_rng(rng)
}

/// Generates a keypair `(PrivateKey, PublicKey)` based on an RNG.
pub fn generate_keypair_from_rng<R>(rng: &mut R) -> (PrivateKey, PublicKey)
where
    R: RngCore + CryptoRng,
{
    let keypair = ed25519_dalek::Keypair::generate(rng);
    (
        PrivateKey {
            value: keypair.secret,
        },
        PublicKey {
            value: keypair.public,
        },
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
) -> (PrivateKey, PublicKey) {
    let mut rng = EntropyRng::new();
    let mut seed_from_rng = [0u8; ed25519_dalek::SECRET_KEY_LENGTH];
    rng.fill_bytes(&mut seed_from_rng);

    let mut final_seed = seed.to_vec();
    final_seed.extend_from_slice(&seed_from_rng);

    derive_keypair_from_seed(salt, &final_seed, app_info)
}

impl Clone for PrivateKey {
    fn clone(&self) -> Self {
        let encoded_privkey: Vec<u8> = serialize(&self.value).unwrap();
        let temp = deserialize::<::ed25519_dalek::SecretKey>(&encoded_privkey).unwrap();
        PrivateKey { value: temp }
    }
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &PrivateKey) -> bool {
        let encoded_privkey: Vec<u8> = serialize(&self.value).unwrap();
        let other_encoded_privkey: Vec<u8> = serialize(&other.value).unwrap();
        encoded_privkey == other_encoded_privkey
    }
}

impl Eq for PrivateKey {}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<elided secret>")
    }
}

// ed25519_dalek doesn't implement Hash, which we need to put signatures into
// containers. For now, the derive_hash_xor_eq has no impact.
impl Hash for PublicKey {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey: Vec<u8> = serialize(&self.value).unwrap();
        state.write(&encoded_pubkey);
    }
}

impl Hash for Signature {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature: Vec<u8> = serialize(&self.value).unwrap();
        state.write(&encoded_signature);
    }
}

impl PartialEq<PublicKey> for PublicKey {
    fn eq(&self, other: &PublicKey) -> bool {
        serialize(&self.value).unwrap() == serialize(&other.value).unwrap()
    }
}

impl Eq for PublicKey {}

impl PartialEq<Signature> for Signature {
    fn eq(&self, other: &Signature) -> bool {
        serialize(&self.value).unwrap() == serialize(&other.value).unwrap()
    }
}

impl Eq for Signature {}

impl PublicKey {
    /// The length of the public key in bytes.
    pub const LENGTH: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;

    /// Obtain a public key from a slice.
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        match ed25519_dalek::PublicKey::from_bytes(data) {
            Ok(key) => Ok(PublicKey { value: key }),
            Err(err) => bail!("Public key decode error: {}", err),
        }
    }

    /// Convert the public key into a slice.
    pub fn to_slice(&self) -> [u8; Self::LENGTH] {
        let mut out = [0u8; Self::LENGTH];
        let temp = self.value.as_bytes();
        out.copy_from_slice(&temp[..]);
        out
    }
}

fn public_key_strategy() -> impl Strategy<Value = PublicKey> {
    any::<[u8; 32]>()
        .prop_map(|seed| {
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            let (_, public_key) = generate_keypair_for_testing(&mut rng);
            public_key
        })
        .no_shrink()
}

impl Arbitrary for PublicKey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        public_key_strategy().boxed()
    }
}

impl From<&PrivateKey> for PublicKey {
    fn from(private_key: &PrivateKey) -> Self {
        let public_key = (&private_key.value).into();
        Self { value: public_key }
    }
}

impl Signature {
    /// Obtains a signature from a byte representation
    pub fn from_compact(data: &[u8]) -> Result<Self> {
        match ed25519_dalek::Signature::from_bytes(data) {
            Ok(sig) => Ok(Signature { value: sig }),
            Err(_error) => bail!("error"),
        }
    }

    /// Converts the signature to its byte representation
    pub fn to_compact(&self) -> [u8; ed25519_dalek::SIGNATURE_LENGTH] {
        let mut out = [0u8; ed25519_dalek::SIGNATURE_LENGTH];
        let temp = self.value.to_bytes();
        out.copy_from_slice(&temp);
        out
    }

    // Check for malleable signatures. This method ensures that the S part is of canonical form
    // and R does not lie on a small group (S and R as defined in RFC 8032).
    fn is_valid(&self) -> Result<()> {
        let bytes = self.to_compact();

        let mut s_bits: [u8; 32] = [0u8; 32];
        s_bits.copy_from_slice(&bytes[32..]);

        // Check if S is of canonical form.
        // We actually test if S < order_of_the_curve to capture malleable signatures.
        let s = Scalar::from_canonical_bytes(s_bits);
        if s == None {
            bail!(
                "Non canonical signature detected: As mentioned in RFC 8032, the 'S' part of the \
                signature should be smaller than the curve order. Consider reducing 'S' by mod 'L', \
                where 'L' is the order of the ed25519 curve.");
        }

        // Check if the R lies on a small subgroup.
        // Even though the security implications of a small order R are unclear,
        // points of order <= 8 are rejected.
        let mut r_bits: [u8; 32] = [0u8; 32];
        r_bits.copy_from_slice(&bytes[..32]);

        let compressed = curve25519_dalek::edwards::CompressedEdwardsY(r_bits);
        let point = compressed.decompress();

        match point {
            Some(p) => {
                if p.is_small_order() {
                    bail!(
                        "Non canonical signature detected: the 'R' part of the signature, \
                         as defined in RFC 8032, lies on a small subgroup."
                    )
                } else {
                    Ok(())
                }
            }
            None => bail!("Malformed signature detected, the 'R' part of the signature is invalid"),
        }
    }
}

impl ser::Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        ed25519_dalek::SecretKey::serialize(&self.value, serializer)
    }
}

impl ser::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        ed25519_dalek::PublicKey::serialize(&self.value, serializer)
    }
}

impl ser::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> export::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        ed25519_dalek::Signature::serialize(&self.value, serializer)
    }
}

struct PrivateKeyVisitor;

struct PublicKeyVisitor;

struct SignatureVisitor;

impl<'de> de::Visitor<'de> for PrivateKeyVisitor {
    type Value = PrivateKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ed25519_dalek private key in bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<PrivateKey, E>
    where
        E: de::Error,
    {
        match ed25519_dalek::SecretKey::from_bytes(value) {
            Ok(key) => Ok(PrivateKey { value: key }),
            Err(error) => Err(E::custom(error)),
        }
    }
}

impl<'de> de::Visitor<'de> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("public key in bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<PublicKey, E>
    where
        E: de::Error,
    {
        match ed25519_dalek::PublicKey::from_bytes(value) {
            Ok(key) => Ok(PublicKey { value: key }),
            Err(error) => Err(E::custom(error)),
        }
    }
}

impl<'de> de::Visitor<'de> for SignatureVisitor {
    type Value = Signature;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ed25519_dalek signature in compact encoding")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> export::Result<Signature, E>
    where
        E: de::Error,
    {
        match ::ed25519_dalek::Signature::from_bytes(value) {
            Ok(key) => Ok(Signature { value: key }),
            Err(error) => Err(E::custom(error)),
        }
    }
}

impl<'de> de::Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PrivateKeyVisitor {})
    }
}

impl<'de> de::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PublicKeyVisitor {})
    }
}

impl<'de> de::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> export::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(SignatureVisitor {})
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.value.to_bytes()[..]))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.value.to_bytes()[..]))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
