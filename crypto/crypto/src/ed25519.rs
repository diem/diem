// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an API for the PureEdDSA signature scheme over the ed25519 twisted
//! Edwards curve as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
//!
//! Signature verification also checks and rejects non-canonical signatures.
//!
//! # Examples
//!
//! ```
//! use libra_crypto::hash::{CryptoHasher, TestOnlyHasher};
//! use libra_crypto::{
//!     ed25519::*,
//!     traits::{Signature, SigningKey, Uniform},
//! };
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let mut hasher = TestOnlyHasher::default();
//! hasher.write("Test message".as_bytes());
//! let hashed_message = hasher.finish();
//!
//! let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
//! let private_key = Ed25519PrivateKey::generate_for_testing(&mut rng);
//! let public_key: Ed25519PublicKey = (&private_key).into();
//! let signature = private_key.sign_message(&hashed_message);
//! assert!(signature.verify(&hashed_message, &public_key).is_ok());
//! ```
//! **Note**: The above example generates a private key using a private function intended only for
//! testing purposes. Production code should find an alternate means for secure key generation.

use crate::{traits::*, HashValue};
use anyhow::{anyhow, Result};
use core::convert::TryFrom;
use libra_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use std::{cmp::Ordering, fmt};

/// The length of the Ed25519PrivateKey
pub const ED25519_PRIVATE_KEY_LENGTH: usize = ed25519_dalek::SECRET_KEY_LENGTH;
/// The length of the Ed25519PublicKey
pub const ED25519_PUBLIC_KEY_LENGTH: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;
/// The length of the Ed25519Signature
pub const ED25519_SIGNATURE_LENGTH: usize = ed25519_dalek::SIGNATURE_LENGTH;

/// The order of ed25519 as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
const L: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

/// An Ed25519 private key
#[derive(DeserializeKey, SilentDisplay, SilentDebug, SerializeKey)]
pub struct Ed25519PrivateKey(ed25519_dalek::SecretKey);

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(Ed25519PrivateKey: Clone);

/// An Ed25519 public key
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct Ed25519PublicKey(ed25519_dalek::PublicKey);

/// An Ed25519 signature
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct Ed25519Signature(ed25519_dalek::Signature);

impl Ed25519PrivateKey {
    /// Serialize an Ed25519PrivateKey.
    pub fn to_bytes(&self) -> [u8; ED25519_PRIVATE_KEY_LENGTH] {
        self.0.to_bytes()
    }

    /// Deserialize an Ed25519PrivateKey without any validation checks apart from expected key size.
    fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<Ed25519PrivateKey, CryptoMaterialError> {
        match ed25519_dalek::SecretKey::from_bytes(bytes) {
            Ok(dalek_secret_key) => Ok(Ed25519PrivateKey(dalek_secret_key)),
            Err(_) => Err(CryptoMaterialError::DeserializationError),
        }
    }
}

impl Ed25519PublicKey {
    /// Serialize an Ed25519PublicKey.
    pub fn to_bytes(&self) -> [u8; ED25519_PUBLIC_KEY_LENGTH] {
        self.0.to_bytes()
    }

    /// Deserialize an Ed25519PublicKey without any validation checks apart from expected key size.
    pub(crate) fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<Ed25519PublicKey, CryptoMaterialError> {
        match ed25519_dalek::PublicKey::from_bytes(bytes) {
            Ok(dalek_public_key) => Ok(Ed25519PublicKey(dalek_public_key)),
            Err(_) => Err(CryptoMaterialError::DeserializationError),
        }
    }
}

impl Ed25519Signature {
    /// Serialize an Ed25519Signature.
    pub fn to_bytes(&self) -> [u8; ED25519_SIGNATURE_LENGTH] {
        self.0.to_bytes()
    }

    /// Deserialize an Ed25519Signature without any validation checks (malleability)
    /// apart from expected key size.
    pub(crate) fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<Ed25519Signature, CryptoMaterialError> {
        match ed25519_dalek::Signature::from_bytes(bytes) {
            Ok(dalek_signature) => Ok(Ed25519Signature(dalek_signature)),
            Err(_) => Err(CryptoMaterialError::DeserializationError),
        }
    }

    /// Check for correct size and third-party based signature malleability issues.
    /// This method is required to ensure that given a valid signature for some message under some
    /// key, an attacker cannot produce another valid signature for the same message and key.
    ///
    /// According to [RFC8032](https://tools.ietf.org/html/rfc8032), signatures comprise elements
    /// {R, S} and we should enforce that S is of canonical form (smaller than L, where L is the
    /// order of edwards25519 curve group) to prevent signature malleability. Without this check,
    /// one could add a multiple of L into S and still pass signature verification, resulting in
    /// a distinct yet valid signature.
    ///
    /// This method does not check the R component of the signature, because R is hashed during
    /// signing and verification to compute h = H(ENC(R) || ENC(A) || M), which means that a
    /// third-party cannot modify R without being detected.
    ///
    /// Note: It's true that malicious signers can already produce varying signatures by
    /// choosing a different nonce, so this method protects against malleability attacks performed
    /// by a non-signer.
    pub fn check_malleability(bytes: &[u8]) -> std::result::Result<(), CryptoMaterialError> {
        if bytes.len() != ED25519_SIGNATURE_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        if !check_s_lt_l(&bytes[32..]) {
            return Err(CryptoMaterialError::CanonicalRepresentationError);
        }
        Ok(())
    }
}

///////////////////////
// PrivateKey Traits //
///////////////////////

impl PrivateKey for Ed25519PrivateKey {
    type PublicKeyMaterial = Ed25519PublicKey;
}

impl SigningKey for Ed25519PrivateKey {
    type VerifyingKeyMaterial = Ed25519PublicKey;
    type SignatureMaterial = Ed25519Signature;

    fn sign_message(&self, message: &HashValue) -> Ed25519Signature {
        let secret_key: &ed25519_dalek::SecretKey = &self.0;
        let public_key: Ed25519PublicKey = self.into();
        let expanded_secret_key: ed25519_dalek::ExpandedSecretKey =
            ed25519_dalek::ExpandedSecretKey::from(secret_key);
        let sig = expanded_secret_key.sign(message.as_ref(), &public_key.0);
        Ed25519Signature(sig)
    }
}

impl Uniform for Ed25519PrivateKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        Ed25519PrivateKey(ed25519_dalek::SecretKey::generate(rng))
    }
}

impl PartialEq<Self> for Ed25519PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for Ed25519PrivateKey {}

// We could have a distinct kind of validation for the PrivateKey, for
// ex. checking the derived PublicKey is valid?
impl TryFrom<&[u8]> for Ed25519PrivateKey {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PrivateKey. This method will also check for key validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<Ed25519PrivateKey, CryptoMaterialError> {
        // Note that the only requirement is that the size of the key is 32 bytes, something that
        // is already checked during deserialization of ed25519_dalek::SecretKey
        // Also, the underlying ed25519_dalek implementation ensures that the derived public key
        // is safe and it will not lie in a small-order group, thus no extra check for PublicKey
        // validation is required.
        Ed25519PrivateKey::from_bytes_unchecked(bytes)
    }
}

impl Length for Ed25519PrivateKey {
    const LENGTH: usize = ed25519_dalek::SECRET_KEY_LENGTH;
}

impl ValidKey for Ed25519PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

impl Genesis for Ed25519PrivateKey {
    fn genesis() -> Self {
        let mut buf = [0u8; ED25519_PRIVATE_KEY_LENGTH];
        buf[ED25519_PRIVATE_KEY_LENGTH - 1] = 1;
        Self::try_from(buf.as_ref()).unwrap()
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////

// Implementing From<&PrivateKey<...>> allows to derive a public key in a more elegant fashion
impl From<&Ed25519PrivateKey> for Ed25519PublicKey {
    fn from(secret_key: &Ed25519PrivateKey) -> Self {
        let secret: &ed25519_dalek::SecretKey = &secret_key.0;
        let public: ed25519_dalek::PublicKey = secret.into();
        Ed25519PublicKey(public)
    }
}

// We deduce PublicKey from this
impl PublicKey for Ed25519PublicKey {
    type PrivateKeyMaterial = Ed25519PrivateKey;
}

impl std::hash::Hash for Ed25519PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

// Those are required by the implementation of hash above
impl PartialEq for Ed25519PublicKey {
    fn eq(&self, other: &Ed25519PublicKey) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for Ed25519PublicKey {}

// We deduce VerifyingKey from pointing to the signature material
// we get the ability to do `pubkey.validate(msg, signature)`
impl VerifyingKey for Ed25519PublicKey {
    type SigningKeyMaterial = Ed25519PrivateKey;
    type SignatureMaterial = Ed25519Signature;
}

impl fmt::Display for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0.as_bytes()))
    }
}

impl fmt::Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519PublicKey({})", self)
    }
}

impl TryFrom<&[u8]> for Ed25519PublicKey {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PublicKey. This method will also check for key validity, for instance
    ///  it will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<Ed25519PublicKey, CryptoMaterialError> {
        // We need to access the Edwards point which is not directly accessible from
        // ed25519_dalek::PublicKey, so we need to do some custom deserialization.
        if bytes.len() != ED25519_PUBLIC_KEY_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let mut bits = [0u8; ED25519_PUBLIC_KEY_LENGTH];
        bits.copy_from_slice(&bytes[..ED25519_PUBLIC_KEY_LENGTH]);

        let compressed = curve25519_dalek::edwards::CompressedEdwardsY(bits);
        let point = compressed
            .decompress()
            .ok_or(CryptoMaterialError::DeserializationError)?;

        // Check if the point lies on a small subgroup. This is required
        // when using curves with a small cofactor (in ed25519, cofactor = 8).
        if point.is_small_order() {
            return Err(CryptoMaterialError::SmallSubgroupError);
        }

        // Unfortunately, tuple struct `PublicKey` is private so we cannot
        // Ok(Ed25519PublicKey(ed25519_dalek::PublicKey(compressed, point)))
        // and we have to again invoke deserialization.
        Ed25519PublicKey::from_bytes_unchecked(bytes)
    }
}

impl Length for Ed25519PublicKey {
    const LENGTH: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;
}

impl ValidKey for Ed25519PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

//////////////////////
// Signature Traits //
//////////////////////

impl Signature for Ed25519Signature {
    type VerifyingKeyMaterial = Ed25519PublicKey;
    type SigningKeyMaterial = Ed25519PrivateKey;

    /// Checks that `self` is valid for `message` using `public_key`.
    fn verify(&self, message: &HashValue, public_key: &Ed25519PublicKey) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in move
    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &Ed25519PublicKey) -> Result<()> {
        Ed25519Signature::check_malleability(&self.to_bytes())?;

        public_key
            .0
            .verify(message, &self.0)
            .map_err(|e| anyhow!("{}", e))
            .and(Ok(()))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// Batch signature verification as described in the original EdDSA article
    /// by Bernstein et al. "High-speed high-security signatures". Current implementation works for
    /// signatures on the same message and it checks for malleability.
    fn batch_verify_signatures(
        message: &HashValue,
        keys_and_signatures: Vec<(Self::VerifyingKeyMaterial, Self)>,
    ) -> Result<()> {
        for (_, sig) in keys_and_signatures.iter() {
            Ed25519Signature::check_malleability(&sig.to_bytes())?
        }
        let batch_argument = keys_and_signatures
            .into_iter()
            .map(|(key, signature)| (key.0, signature.0));
        let (dalek_public_keys, dalek_signatures): (Vec<_>, Vec<_>) = batch_argument.unzip();
        let message_ref = &message.as_ref()[..];
        // The original batching algorithm works for different messages and it expects as many
        // messages as the number of signatures. In our case, we just populate the same
        // message to meet dalek's api requirements.
        let messages = vec![message_ref; dalek_signatures.len()];
        ed25519_dalek::verify_batch(&messages[..], &dalek_signatures[..], &dalek_public_keys[..])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }
}

impl Length for Ed25519Signature {
    const LENGTH: usize = ED25519_SIGNATURE_LENGTH;
}

impl ValidKey for Ed25519Signature {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

impl std::hash::Hash for Ed25519Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

impl TryFrom<&[u8]> for Ed25519Signature {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<Ed25519Signature, CryptoMaterialError> {
        Ed25519Signature::check_malleability(bytes)?;
        Ed25519Signature::from_bytes_unchecked(bytes)
    }
}

// Those are required by the implementation of hash above
impl PartialEq for Ed25519Signature {
    fn eq(&self, other: &Ed25519Signature) -> bool {
        self.to_bytes().as_ref() == other.to_bytes().as_ref()
    }
}

impl Eq for Ed25519Signature {}

impl fmt::Display for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0.to_bytes()[..]))
    }
}

impl fmt::Debug for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519Signature({})", self)
    }
}

/// Check if S < L to capture invalid signatures.
fn check_s_lt_l(s: &[u8]) -> bool {
    for i in (0..32).rev() {
        match s[i].cmp(&L[i]) {
            Ordering::Less => return true,
            Ordering::Greater => return false,
            _ => {}
        }
    }
    // As this stage S == L which implies a non canonical S.
    false
}

//////////////////////////
// Compatibility Traits //
//////////////////////////

/// Those transitory traits are meant to help with the progressive
/// migration of the code base to the crypto module and will
/// disappear after
pub mod compat {
    use crate::ed25519::*;
    #[cfg(feature = "fuzzing")]
    use proptest::strategy::LazyJust;
    #[cfg(feature = "fuzzing")]
    use proptest::{prelude::*, strategy::Strategy};

    #[cfg(any(test, feature = "cloneable-private-keys"))]
    impl Clone for Ed25519PrivateKey {
        fn clone(&self) -> Self {
            let serialized: &[u8] = &(self.to_bytes());
            Ed25519PrivateKey::try_from(serialized).unwrap()
        }
    }

    use crate::Uniform;
    use rand::{rngs::StdRng, SeedableRng};

    /// Generate an arbitrary key pair, with possible Rng input
    ///
    /// Warning: if you pass in None, this will not return distinct
    /// results every time! Should you want to write non-deterministic
    /// tests, look at libra_config::config_builder::util::get_test_config
    pub fn generate_keypair<'a, T>(opt_rng: T) -> (Ed25519PrivateKey, Ed25519PublicKey)
    where
        T: Into<Option<&'a mut StdRng>> + Sized,
    {
        if let Some(rng_mut_ref) = opt_rng.into() {
            <(Ed25519PrivateKey, Ed25519PublicKey)>::generate_for_testing(rng_mut_ref)
        } else {
            let mut rng = StdRng::from_seed(crate::test_utils::TEST_SEED);
            <(Ed25519PrivateKey, Ed25519PublicKey)>::generate_for_testing(&mut rng)
        }
    }

    /// Used to produce keypairs from a seed for testing purposes
    #[cfg(feature = "fuzzing")]
    pub fn keypair_strategy() -> impl Strategy<Value = (Ed25519PrivateKey, Ed25519PublicKey)> {
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

    /// Generates a well-known keypair `(Ed25519PrivateKey, Ed25519PublicKey)` for special use
    /// in the genesis block. A genesis block is the first block of a blockchain and it is
    /// hardcoded as it's a special case in that it does not reference a previous block.
    pub fn generate_genesis_keypair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
        let mut buf = [0u8; ED25519_PRIVATE_KEY_LENGTH];
        buf[ED25519_PRIVATE_KEY_LENGTH - 1] = 1;
        let private_key = Ed25519PrivateKey::try_from(&buf[..]).unwrap();
        let public_key = (&private_key).into();
        (private_key, public_key)
    }

    #[cfg(feature = "fuzzing")]
    impl Arbitrary for Ed25519PublicKey {
        type Parameters = ();
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            LazyJust::new(|| generate_keypair(None).1).boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}
