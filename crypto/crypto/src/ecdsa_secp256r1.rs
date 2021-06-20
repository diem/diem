// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an API for the ECDSA signature scheme over the secp256r1 twisted curve as
//! as specified in [FIPS 186-4](https://csrc.nist.gov/publications/detail/fips/186/4/final) (Digital Signature Standard).
//!
//!
//! # Examples
#![allow(clippy::integer_arithmetic)]

use crate::{
    hash::{CryptoHash, CryptoHasher},
    traits::*,
};
use anyhow::{anyhow, Result};
use std::convert::{TryFrom, TryInto};
use diem_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use mirai_annotations::*;
use p256::{
    ecdsa::{SigningKey as P256SigningKey, Signature as P256Signature, signature::Signer, VerifyingKey as P256VerifyingKey, signature::Verifier}
};
use serde::Serialize;
use std::{cmp::Ordering, fmt};

/// The length of the EcdsaPublicKey
pub const ECDSA_SECP256R1_PRIVATE_KEY_LENGTH: usize = 32;
/// The length of the EcdsaPublicKey
pub const ECDSA_SECP256R1_PUBLIC_KEY_LENGTH: usize = 33;
/// The length of the EcdsaSignature
pub const ECDSA_SECP256R1_SIGNATURE_LENGTH: usize = 64;

/// An ECDSA secp256r1 private key
#[derive(DeserializeKey, SerializeKey, SilentDebug, SilentDisplay)]
pub struct EcdsaPrivateKey(Vec<u8>);

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(EcdsaPrivateKey: Clone);

#[cfg(any(test, feature = "cloneable-private-keys"))]
impl Clone for EcdsaPrivateKey {
    fn clone(&self) -> Self {
        EcdsaPrivateKey(self.0.clone())
    }
}

/// An ECDSA secp256r1 public key
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct EcdsaPublicKey(Vec<u8>);

#[cfg(mirai)]
use crate::tags::ValidatedPublicKeyTag;
#[cfg(not(mirai))]
struct ValidatedPublicKeyTag {}

/// An ECDSA secp256r1 signature
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct EcdsaSignature(Vec<u8>);

impl EcdsaPrivateKey {
    /// The length of the EcdsaPrivateKey
    pub const LENGTH: usize = ECDSA_SECP256R1_PRIVATE_KEY_LENGTH;

    /// Serialize an EcdsaPrivateKey.
    pub fn to_bytes(&self) -> [u8; EcdsaPrivateKey::LENGTH] {
        // Converting from Vec to array will not normally fail, and if it does it's better to panic.
        self.0.clone().try_into().unwrap()
    }

    /// Deserialize an EcdsaPrivateKey without any validation checks apart from expected key size.
    fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<EcdsaPrivateKey, CryptoMaterialError> {
        if bytes.len() != ECDSA_SECP256R1_PRIVATE_KEY_LENGTH {
            Err(CryptoMaterialError::DeserializationError)
        } else {
            Ok(EcdsaPrivateKey(bytes.to_vec()))
        }
    }

    /// Private function aimed at minimizing code duplication between sign
    /// methods of the SigningKey implementation. This should remain private.
    fn sign_arbitrary_message(&self, message: &[u8]) -> EcdsaSignature {
        let signing_key = P256SigningKey::from_bytes(&self.0).unwrap();
        let signature: P256Signature = signing_key.sign(message);
        EcdsaSignature(to_compressed_signature_bytes(&signature))
    }
}

impl EcdsaPublicKey {
    /// Serialize an EcdsaPublicKey.
    pub fn to_bytes(&self) -> [u8; ECDSA_SECP256R1_PUBLIC_KEY_LENGTH] { self.0.clone().try_into().unwrap() }

    /// Deserialize an EcdsaPublicKey without any validation checks apart from expected key size.
    pub(crate) fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<EcdsaPublicKey, CryptoMaterialError> {
        if bytes.len() != ECDSA_SECP256R1_PUBLIC_KEY_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError)
        }
        Ok(EcdsaPublicKey(bytes.to_vec()))
    }
}

impl EcdsaSignature {
    /// The length of the EcdsaSignature
    pub const LENGTH: usize = ECDSA_SECP256R1_SIGNATURE_LENGTH;

    /// Serialize an EcdsaSignature.
    pub fn to_bytes(&self) -> [u8; ECDSA_SECP256R1_SIGNATURE_LENGTH] {
        self.0.clone().try_into().unwrap()
    }

    /// Deserialize an EcdsaSignature without any validation checks
    /// apart from expected key size.
    pub(crate) fn from_bytes_unchecked(
        bytes: &[u8],
    ) -> std::result::Result<EcdsaSignature, CryptoMaterialError> {
        if bytes.len() != ECDSA_SECP256R1_SIGNATURE_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError)
        }
        Ok(EcdsaSignature(bytes.to_vec()))
    }

    /// return an all-zero signature (for test only)
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn dummy_signature() -> Self {
        Self::from_bytes_unchecked(&[0u8; Self::LENGTH]).unwrap()
    }
}

///////////////////////
// PrivateKey Traits //
///////////////////////

impl PrivateKey for EcdsaPrivateKey {
    type PublicKeyMaterial = EcdsaPublicKey;
}

impl SigningKey for EcdsaPrivateKey {
    type VerifyingKeyMaterial = EcdsaPublicKey;
    type SignatureMaterial = EcdsaSignature;

    fn sign<T: CryptoHash + Serialize>(&self, message: &T) -> EcdsaSignature {
        let mut bytes = <T::Hasher as CryptoHasher>::seed().to_vec();
        bcs::serialize_into(&mut bytes, &message)
            .map_err(|_| CryptoMaterialError::SerializationError)
            .expect("Serialization of signable material should not fail.");
        EcdsaPrivateKey::sign_arbitrary_message(&self, bytes.as_ref())
    }

    #[cfg(any(test, feature = "fuzzing"))]
    fn sign_arbitrary_message(&self, message: &[u8]) -> EcdsaSignature {
        EcdsaPrivateKey::sign_arbitrary_message(self, message)
    }
}

impl Uniform for EcdsaPrivateKey {
    fn generate<R>(rng: &mut R) -> Self
        where
            R: ::rand::RngCore + ::rand::CryptoRng,
    {
        let signing_key = P256SigningKey::random(rng);
        EcdsaPrivateKey(signing_key.to_bytes().to_vec())
    }
}

impl PartialEq<Self> for EcdsaPrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for EcdsaPrivateKey {}

// We could have a distinct kind of validation for the PrivateKey, for
// ex. checking the derived PublicKey is valid?
impl TryFrom<&[u8]> for EcdsaPrivateKey {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PrivateKey. This method will also check for key validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<EcdsaPrivateKey, CryptoMaterialError> {
        if bytes.len() != EcdsaPrivateKey::LENGTH {
            return Err(CryptoMaterialError::WrongLengthError)
        }
        Ok(EcdsaPrivateKey(bytes.to_vec()))
    }
}

impl Length for EcdsaPrivateKey {
    fn length(&self) -> usize {
        Self::LENGTH
    }
}

impl ValidCryptoMaterial for EcdsaPrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

impl Genesis for EcdsaPrivateKey {
    fn genesis() -> Self {
        let mut buf = [0u8; EcdsaPrivateKey::LENGTH];
        buf[EcdsaPrivateKey::LENGTH - 1] = 1;
        Self::try_from(buf.as_ref()).unwrap()
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////

// Implementing From<&PrivateKey<...>> allows to derive a public key in a more elegant fashion
impl From<&EcdsaPrivateKey> for EcdsaPublicKey {
    fn from(private_key: &EcdsaPrivateKey) -> Self {
        let signing_key = P256SigningKey::from_bytes(&private_key.0).unwrap();
        let verifying_key = P256VerifyingKey::from(&signing_key);
        EcdsaPublicKey(verifying_key.to_encoded_point(true).to_bytes().to_vec())
    }
}

// We deduce PublicKey from this
impl PublicKey for EcdsaPublicKey {
    type PrivateKeyMaterial = EcdsaPrivateKey;
}

impl std::hash::Hash for EcdsaPublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

// Those are required by the implementation of hash above
impl PartialEq for EcdsaPublicKey {
    fn eq(&self, other: &EcdsaPublicKey) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for EcdsaPublicKey {}

// We deduce VerifyingKey from pointing to the signature material
// we get the ability to do `pubkey.validate(msg, signature)`
impl VerifyingKey for EcdsaPublicKey {
    type SigningKeyMaterial = EcdsaPrivateKey;
    type SignatureMaterial = EcdsaSignature;
}

impl fmt::Display for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Debug for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EcdsaPublicKey({})", self)
    }
}

impl TryFrom<&[u8]> for EcdsaPublicKey {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PublicKey. This method will also check for key validity, for instance
    ///  it will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<EcdsaPublicKey, CryptoMaterialError> {
        let public_key = EcdsaPublicKey::from_bytes_unchecked(bytes)?;
        add_tag!(&public_key, ValidatedPublicKeyTag); // This key has gone through validity checks.
        Ok(public_key)
    }
}

impl Length for EcdsaPublicKey {
    fn length(&self) -> usize {
        ECDSA_SECP256R1_PUBLIC_KEY_LENGTH
    }
}

impl ValidCryptoMaterial for EcdsaPublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

//////////////////////
// Signature Traits //
//////////////////////

impl Signature for EcdsaSignature {
    type VerifyingKeyMaterial = EcdsaPublicKey;
    type SigningKeyMaterial = EcdsaPrivateKey;

    /// Verifies that the provided signature is valid for the provided
    /// message, according to the RFC8032 algorithm. This strict verification performs the
    /// recommended check of 5.1.7 §3, on top of the required RFC8032 verifications.
    fn verify<T: CryptoHash + Serialize>(
        &self,
        message: &T,
        public_key: &EcdsaPublicKey,
    ) -> Result<()> {
        let mut bytes = <T::Hasher as CryptoHasher>::seed().to_vec();
        bcs::serialize_into(&mut bytes, &message)
            .map_err(|_| CryptoMaterialError::SerializationError)?;
        Self::verify_arbitrary_msg(self, &bytes, public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in move
    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &EcdsaPublicKey) -> Result<()> {
        let verifying_key = P256VerifyingKey::from_sec1_bytes(&public_key.0).unwrap();
        let signature = P256Signature::try_from((&self.0).as_slice()).unwrap();
        verifying_key.verify(message, &signature)
            .map_err(|e| anyhow!("{}", e))
            .and(Ok(()))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl Length for EcdsaSignature {
    fn length(&self) -> usize {
        ECDSA_SECP256R1_SIGNATURE_LENGTH
    }
}

impl ValidCryptoMaterial for EcdsaSignature {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

impl std::hash::Hash for EcdsaSignature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

impl TryFrom<&[u8]> for EcdsaSignature {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<EcdsaSignature, CryptoMaterialError> {
        EcdsaSignature::from_bytes_unchecked(bytes)
    }
}

// Those are required by the implementation of hash above
impl PartialEq for EcdsaSignature {
    fn eq(&self, other: &EcdsaSignature) -> bool {
        self.to_bytes()[..] == other.to_bytes()[..]
    }
}

impl Eq for EcdsaSignature {}

impl fmt::Display for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Debug for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EcdsaSignature({})", self)
    }
}

// --- Utilities ---
fn to_compressed_signature_bytes(signature: &P256Signature) -> Vec<u8>{
    let mut sig_bytes = [0u8; 64];
    sig_bytes[..32].copy_from_slice(&signature.r().to_bytes());
    sig_bytes[32..].copy_from_slice(&signature.s().to_bytes());
    sig_bytes.to_vec()
}

#[cfg(any(test, feature = "fuzzing"))]
use crate::test_utils::{self, KeyPair};

/// Produces a uniformly random ecdsa keypair from a seed
#[cfg(any(test, feature = "fuzzing"))]
pub fn keypair_strategy() -> impl Strategy<Value = KeyPair<EcdsaPrivateKey, EcdsaPublicKey>> {
    test_utils::uniform_keypair_strategy::<EcdsaPrivateKey, EcdsaPublicKey>()
}

#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use rand_core::OsRng;

#[cfg(any(test, feature = "fuzzing"))]
impl proptest::arbitrary::Arbitrary for EcdsaPublicKey {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        crate::test_utils::uniform_keypair_strategy::<EcdsaPrivateKey, EcdsaPublicKey>()
            .prop_map(|v| v.public_key)
            .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

#[test]
fn sign_verify() {
    let message = b"Hello world";
    let message2 = b"Hello world2";
    let key_bytes = b"12345678901234567890123456789012";
    let private_key = EcdsaPrivateKey::from_bytes_unchecked(&key_bytes.to_vec().as_slice()).unwrap();
    let public_key = EcdsaPublicKey::from(&private_key);
    let signature = private_key.sign_arbitrary_message(&message.to_vec().as_slice());
    assert!(signature.verify_arbitrary_msg(&message.to_vec().as_slice(), &public_key).is_ok());
    assert!(signature.verify_arbitrary_msg(&message2.to_vec().as_slice(), &public_key).is_err());
}
