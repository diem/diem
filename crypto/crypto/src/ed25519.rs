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
//!     ed25519,
//!     TSignature, SigningKey, Uniform,
//! };
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let mut hasher = TestOnlyHasher::default();
//! hasher.write("Test message".as_bytes());
//! let hashed_message = hasher.finish();
//!
//! let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
//! let private_key = ed25519::SigningKey::generate(&mut rng);
//! let public_key: ed25519::VerifyingKey = (&private_key).into();
//! let signature = private_key.sign_message(&hashed_message);
//! assert!(signature.verify(&hashed_message, &public_key).is_ok());
//! ```
//! **Note**: The above example generates a private key using a private function intended only for
//! testing purposes. Production code should find an alternate means for secure key generation.

use crate::{
    traits::{self, CryptoMaterialError, Genesis, Length, Uniform, ValidKey, ValidKeyStringExt},
    HashValue,
};
use anyhow::{anyhow, Result};
use core::convert::TryFrom;
use libra_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use std::{cmp::Ordering, fmt};

/// The length of the SigningKey
pub const PRIVATE_KEY_LENGTH: usize = ed25519_dalek::SECRET_KEY_LENGTH;
/// The length of the VerifyingKey
pub const PUBLIC_KEY_LENGTH: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;
/// The length of the Signature
pub const SIGNATURE_LENGTH: usize = ed25519_dalek::SIGNATURE_LENGTH;

/// The order of ed25519 as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
const L: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

/// An Ed25519 private key
#[derive(DeserializeKey, SilentDisplay, SilentDebug, SerializeKey)]
pub struct SigningKey(ed25519_dalek::SecretKey);

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(SigningKey: Clone);

#[cfg(any(test, feature = "cloneable-private-keys"))]
impl Clone for SigningKey {
    fn clone(&self) -> Self {
        let serialized: &[u8] = &(self.to_bytes());
        SigningKey::try_from(serialized).unwrap()
    }
}

/// An Ed25519 public key
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct VerifyingKey(ed25519_dalek::PublicKey);

/// An Ed25519 signature
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct Signature(ed25519_dalek::Signature);

impl SigningKey {
    /// The length of the SigningKey
    pub const LENGTH: usize = ed25519_dalek::SECRET_KEY_LENGTH;

    /// Serialize an SigningKey.
    pub fn to_bytes(&self) -> [u8; PRIVATE_KEY_LENGTH] {
        self.0.to_bytes()
    }

    /// Deserialize an SigningKey without any validation checks apart from expected key size.
    fn from_bytes_unchecked(bytes: &[u8]) -> std::result::Result<SigningKey, CryptoMaterialError> {
        match ed25519_dalek::SecretKey::from_bytes(bytes) {
            Ok(dalek_secret_key) => Ok(SigningKey(dalek_secret_key)),
            Err(_) => Err(CryptoMaterialError::DeserializationError),
        }
    }
}

impl VerifyingKey {
    /// Serialize an VerifyingKey.
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.0.to_bytes()
    }

    /// Deserialize an VerifyingKey without any validation checks apart from expected key size.
    pub(crate) fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<VerifyingKey, CryptoMaterialError> {
        match ed25519_dalek::PublicKey::from_bytes(bytes) {
            Ok(dalek_public_key) => Ok(VerifyingKey(dalek_public_key)),
            Err(_) => Err(CryptoMaterialError::DeserializationError),
        }
    }
}

impl Signature {
    /// The length of the Signature
    pub const LENGTH: usize = ed25519_dalek::SIGNATURE_LENGTH;

    /// Serialize an Signature.
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.0.to_bytes()
    }

    /// Deserialize an Signature without any validation checks (malleability)
    /// apart from expected key size.
    pub(crate) fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<Signature, CryptoMaterialError> {
        match ed25519_dalek::Signature::from_bytes(bytes) {
            Ok(dalek_signature) => Ok(Signature(dalek_signature)),
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
        if bytes.len() != SIGNATURE_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        if !check_s_lt_l(&bytes[32..]) {
            return Err(CryptoMaterialError::CanonicalRepresentationError);
        }
        Ok(())
    }
}

///////////////////////
// SigningKey Traits //
///////////////////////

impl traits::PrivateKey for SigningKey {
    type PublicKeyMaterial = VerifyingKey;
}

impl traits::SigningKey for SigningKey {
    type VerifyingKeyMaterial = VerifyingKey;
    type SignatureMaterial = Signature;

    fn sign_message(&self, message: &HashValue) -> Signature {
        let secret_key: &ed25519_dalek::SecretKey = &self.0;
        let public_key: VerifyingKey = self.into();
        let expanded_secret_key: ed25519_dalek::ExpandedSecretKey =
            ed25519_dalek::ExpandedSecretKey::from(secret_key);
        let sig = expanded_secret_key.sign(message.as_ref(), &public_key.0);
        Signature(sig)
    }
}

impl Uniform for SigningKey {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        SigningKey(ed25519_dalek::SecretKey::generate(rng))
    }
}

impl PartialEq<Self> for SigningKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for SigningKey {}

// We could have a distinct kind of validation for the SigningKey, for
// ex. checking the derived VerifyingKey is valid?
impl TryFrom<&[u8]> for SigningKey {
    type Error = CryptoMaterialError;

    /// Deserialize an SigningKey. This method will also check for key validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<SigningKey, CryptoMaterialError> {
        // Note that the only requirement is that the size of the key is 32 bytes, something that
        // is already checked during deserialization of ed25519_dalek::SecretKey
        // Also, the underlying ed25519_dalek implementation ensures that the derived public key
        // is safe and it will not lie in a small-order group, thus no extra check for VerifyingKey
        // validation is required.
        SigningKey::from_bytes_unchecked(bytes)
    }
}

impl Length for SigningKey {
    fn length(&self) -> usize {
        Self::LENGTH
    }
}

impl ValidKey for SigningKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

impl Genesis for SigningKey {
    fn genesis() -> Self {
        let mut buf = [0u8; PRIVATE_KEY_LENGTH];
        buf[PRIVATE_KEY_LENGTH - 1] = 1;
        Self::try_from(buf.as_ref()).unwrap()
    }
}

//////////////////////
// VerifyingKey Traits //
//////////////////////

// Implementing From<&SigningKey<...>> allows to derive a public key in a more elegant fashion
impl From<&SigningKey> for VerifyingKey {
    fn from(private_key: &SigningKey) -> Self {
        let secret: &ed25519_dalek::SecretKey = &private_key.0;
        let public: ed25519_dalek::PublicKey = secret.into();
        VerifyingKey(public)
    }
}

// We deduce VerifyingKey from this
impl traits::PublicKey for VerifyingKey {
    type PrivateKeyMaterial = SigningKey;
}

impl std::hash::Hash for VerifyingKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

// Those are required by the implementation of hash above
impl PartialEq for VerifyingKey {
    fn eq(&self, other: &VerifyingKey) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for VerifyingKey {}

// We deduce VerifyingKey from pointing to the signature material
// we get the ability to do `pubkey.validate(msg, signature)`
impl traits::VerifyingKey for VerifyingKey {
    type SigningKeyMaterial = SigningKey;
    type SignatureMaterial = Signature;
}

impl fmt::Display for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0.as_bytes()))
    }
}

impl fmt::Debug for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VerifyingKey({})", self)
    }
}

impl TryFrom<&[u8]> for VerifyingKey {
    type Error = CryptoMaterialError;

    /// Deserialize an VerifyingKey. This method will also check for key validity, for instance
    ///  it will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<VerifyingKey, CryptoMaterialError> {
        // We need to access the Edwards point which is not directly accessible from
        // ed25519_dalek::PublicKey, so we need to do some custom deserialization.
        if bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let mut bits = [0u8; PUBLIC_KEY_LENGTH];
        bits.copy_from_slice(&bytes[..PUBLIC_KEY_LENGTH]);

        let compressed = curve25519_dalek::edwards::CompressedEdwardsY(bits);
        let point = compressed
            .decompress()
            .ok_or(CryptoMaterialError::DeserializationError)?;

        // Check if the point lies on a small subgroup. This is required
        // when using curves with a small cofactor (in ed25519, cofactor = 8).
        if point.is_small_order() {
            return Err(CryptoMaterialError::SmallSubgroupError);
        }

        // Unfortunately, tuple struct `VerifyingKey` is private so we cannot
        // Ok(VerifyingKey(ed25519_dalek::PublicKey(compressed, point)))
        // and we have to again invoke deserialization.
        VerifyingKey::from_bytes_unchecked(bytes)
    }
}

impl Length for VerifyingKey {
    fn length(&self) -> usize {
        PUBLIC_KEY_LENGTH
    }
}

impl ValidKey for VerifyingKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

//////////////////////
// Signature Traits //
//////////////////////

impl traits::Signature for Signature {
    type VerifyingKeyMaterial = VerifyingKey;
    type SigningKeyMaterial = SigningKey;

    /// Checks that `self` is valid for `message` using `public_key`.
    fn verify(&self, message: &HashValue, public_key: &VerifyingKey) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in move
    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &VerifyingKey) -> Result<()> {
        Signature::check_malleability(&self.to_bytes())?;

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
            Signature::check_malleability(&sig.to_bytes())?
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

impl Length for Signature {
    fn length(&self) -> usize {
        SIGNATURE_LENGTH
    }
}

impl ValidKey for Signature {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

impl std::hash::Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<Signature, CryptoMaterialError> {
        Signature::check_malleability(bytes)?;
        Signature::from_bytes_unchecked(bytes)
    }
}

// Those are required by the implementation of hash above
impl PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        self.to_bytes().as_ref() == other.to_bytes().as_ref()
    }
}

impl Eq for Signature {}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0.to_bytes()[..]))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({})", self)
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

#[cfg(any(test, feature = "fuzzing"))]
use crate::test_utils::{self, KeyPair};

/// Produces a uniformly random ed25519 keypair from a seed
#[cfg(any(test, feature = "fuzzing"))]
pub fn keypair_strategy() -> impl Strategy<Value = KeyPair<SigningKey, VerifyingKey>> {
    test_utils::uniform_keypair_strategy::<SigningKey, VerifyingKey>()
}

#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "fuzzing"))]
impl proptest::arbitrary::Arbitrary for VerifyingKey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        crate::test_utils::uniform_keypair_strategy::<SigningKey, VerifyingKey>()
            .prop_map(|v| v.public_key)
            .boxed()
    }
}
