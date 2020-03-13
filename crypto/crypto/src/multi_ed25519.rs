// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an API for the accountable threshold multi-sig PureEdDSA signature scheme
//! over the ed25519 twisted Edwards curve as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
//!
//! Signature verification also checks and rejects non-canonical signatures.

use crate::{
    ed25519::{
        Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PRIVATE_KEY_LENGTH,
        ED25519_PUBLIC_KEY_LENGTH, ED25519_SIGNATURE_LENGTH,
    },
    traits::*,
    HashValue,
};
use anyhow::{anyhow, Result};
use bit_vec::BitVec;
use core::convert::TryFrom;
use libra_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use once_cell::sync::Lazy;
use rand::Rng;
use std::fmt;

const MAX_NUM_OF_KEYS: usize = 32;
const BITVEC_NUM_OF_BYTES: usize = MAX_NUM_OF_KEYS / 8;
static BITVEC_FIRST_KEY_SET: Lazy<BitVec> =
    Lazy::new(|| BitVec::from_bytes(&[0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]));

/// Vector of private keys in the multi-key Ed25519 structure along with the threshold.
#[derive(DeserializeKey, Eq, PartialEq, SilentDisplay, SilentDebug, SerializeKey)]
pub struct MultiEd25519PrivateKey {
    private_keys: Vec<Ed25519PrivateKey>,
    threshold: u8,
}

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(MultiEd25519PrivateKey: Clone);

/// Vector of public keys in the multi-key Ed25519 structure along with the threshold.
#[derive(Clone, DeserializeKey, Eq, PartialEq, SerializeKey)]
pub struct MultiEd25519PublicKey {
    public_keys: Vec<Ed25519PublicKey>,
    threshold: u8,
}

/// Vector of the multi-key signatures along with their index required to map signatures with
/// their corresponding public keys.
#[derive(Clone, DeserializeKey, Eq, PartialEq, SerializeKey)]
pub struct MultiEd25519Signature {
    signatures: Vec<Ed25519Signature>,
    bit_vector: BitVec,
}

impl MultiEd25519PrivateKey {
    /// Construct a new MultiEd25519PrivateKey.
    pub fn new(
        private_keys: Vec<Ed25519PrivateKey>,
        threshold: u8,
    ) -> std::result::Result<Self, CryptoMaterialError> {
        let num_of_keys = private_keys.len();
        if threshold == 0 || num_of_keys < threshold as usize {
            Err(CryptoMaterialError::ValidationError)
        } else if num_of_keys > MAX_NUM_OF_KEYS {
            Err(CryptoMaterialError::WrongLengthError)
        } else {
            Ok(MultiEd25519PrivateKey {
                private_keys,
                threshold,
            })
        }
    }

    /// Serialize a MultiEd25519PrivateKey.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.private_keys, self.threshold)
    }
}

impl MultiEd25519PublicKey {
    /// Construct a new MultiEd25519PublicKey.
    /// --- Rules ---
    /// a) threshold cannot be zero.
    /// b) public_keys.len() should be equal to or larger than threshold.
    /// c) support up to MAX_NUM_OF_KEYS public keys.
    pub fn new(
        public_keys: Vec<Ed25519PublicKey>,
        threshold: u8,
    ) -> std::result::Result<Self, CryptoMaterialError> {
        let num_of_keys = public_keys.len();
        if threshold == 0 || num_of_keys < threshold as usize {
            Err(CryptoMaterialError::ValidationError)
        } else if num_of_keys > MAX_NUM_OF_KEYS {
            Err(CryptoMaterialError::WrongLengthError)
        } else {
            Ok(MultiEd25519PublicKey {
                public_keys,
                threshold,
            })
        }
    }

    /// Getter public_keys
    pub fn public_keys(&self) -> &Vec<Ed25519PublicKey> {
        &self.public_keys
    }

    /// Getter threshold
    pub fn threshold(&self) -> &u8 {
        &self.threshold
    }

    /// Serialize a MultiEd25519PublicKey.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.public_keys, self.threshold)
    }
}

///////////////////////
// PrivateKey Traits //
///////////////////////

/// Convenient method to create a MultiEd25519PrivateKey from a single Ed25519PrivateKey.
impl From<&Ed25519PrivateKey> for MultiEd25519PrivateKey {
    fn from(ed_private_key: &Ed25519PrivateKey) -> Self {
        MultiEd25519PrivateKey {
            private_keys: vec![Ed25519PrivateKey::try_from(&ed_private_key.to_bytes()[..]).unwrap()],
            threshold: 1u8,
        }
    }
}

impl PrivateKey for MultiEd25519PrivateKey {
    type PublicKeyMaterial = MultiEd25519PublicKey;
}

impl SigningKey for MultiEd25519PrivateKey {
    type VerifyingKeyMaterial = MultiEd25519PublicKey;
    type SignatureMaterial = MultiEd25519Signature;

    // Sign a message with the minimum amount of keys to meet threshold (starting from left-most keys).
    fn sign_message(&self, message: &HashValue) -> MultiEd25519Signature {
        let mut signatures: Vec<Ed25519Signature> = Vec::with_capacity(self.threshold as usize);
        let mut bit_vector = BitVec::from_elem(MAX_NUM_OF_KEYS, false);

        signatures.extend(
            self.private_keys
                .iter()
                .take(self.threshold as usize)
                .enumerate()
                .map(|(i, item)| {
                    bit_vector.set(i, true);
                    item.sign_message(message)
                }),
        );
        MultiEd25519Signature {
            signatures,
            bit_vector,
        }
    }
}

// Generating a random K out-of N key for testing.
impl Uniform for MultiEd25519PrivateKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let num_of_keys = rng.gen_range(1, MAX_NUM_OF_KEYS + 1);
        let mut private_keys: Vec<Ed25519PrivateKey> = Vec::with_capacity(num_of_keys);
        for _ in 1..=num_of_keys {
            private_keys.push(
                Ed25519PrivateKey::try_from(
                    &ed25519_dalek::SecretKey::generate(rng).to_bytes()[..],
                )
                .unwrap(),
            );
        }
        let threshold = rng.gen_range(1, num_of_keys + 1) as u8;
        MultiEd25519PrivateKey {
            private_keys,
            threshold,
        }
    }
}

impl TryFrom<&[u8]> for MultiEd25519PrivateKey {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PrivateKey. This method will also check for key and threshold validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiEd25519PrivateKey, CryptoMaterialError> {
        if bytes.is_empty() {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let threshold = check_and_get_threshold(bytes, ED25519_PRIVATE_KEY_LENGTH)?;

        let private_keys: Result<Vec<Ed25519PrivateKey>, _> = bytes
            .chunks_exact(ED25519_PRIVATE_KEY_LENGTH)
            .map(Ed25519PrivateKey::try_from)
            .collect();

        private_keys.map(|private_keys| MultiEd25519PrivateKey {
            private_keys,
            threshold,
        })
    }
}

impl Length for MultiEd25519PrivateKey {
    fn length(&self) -> usize {
        self.private_keys.len() * ED25519_PRIVATE_KEY_LENGTH + 1
    }
}

impl ValidKey for MultiEd25519PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Genesis for MultiEd25519PrivateKey {
    fn genesis() -> Self {
        let mut buf = [0u8; ED25519_PRIVATE_KEY_LENGTH];
        buf[ED25519_PRIVATE_KEY_LENGTH - 1] = 1u8;
        MultiEd25519PrivateKey {
            private_keys: vec![Ed25519PrivateKey::try_from(buf.as_ref()).unwrap()],
            threshold: 1u8,
        }
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////

/// Convenient method to create a MultiEd25519PublicKey from a single Ed25519PublicKey.
impl From<&Ed25519PublicKey> for MultiEd25519PublicKey {
    fn from(ed_public_key: &Ed25519PublicKey) -> Self {
        MultiEd25519PublicKey {
            public_keys: vec![ed_public_key.clone()],
            threshold: 1u8,
        }
    }
}

/// Implementing From<&PrivateKey<...>> allows to derive a public key in a more elegant fashion.
impl From<&MultiEd25519PrivateKey> for MultiEd25519PublicKey {
    fn from(private_key: &MultiEd25519PrivateKey) -> Self {
        let public_keys = private_key
            .private_keys
            .iter()
            .map(PrivateKey::public_key)
            .collect();
        MultiEd25519PublicKey {
            public_keys,
            threshold: private_key.threshold,
        }
    }
}

/// We deduce PublicKey from this.
impl PublicKey for MultiEd25519PublicKey {
    type PrivateKeyMaterial = MultiEd25519PrivateKey;
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for MultiEd25519PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

impl TryFrom<&[u8]> for MultiEd25519PublicKey {
    type Error = CryptoMaterialError;

    /// Deserialize a MultiEd25519PublicKey. This method will also check for key and threshold
    /// validity, and will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiEd25519PublicKey, CryptoMaterialError> {
        if bytes.is_empty() {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let threshold = check_and_get_threshold(bytes, ED25519_PUBLIC_KEY_LENGTH)?;
        let public_keys: Result<Vec<Ed25519PublicKey>, _> = bytes
            .chunks_exact(ED25519_PUBLIC_KEY_LENGTH)
            .map(Ed25519PublicKey::try_from)
            .collect();
        public_keys.map(|public_keys| MultiEd25519PublicKey {
            public_keys,
            threshold,
        })
    }
}

/// We deduce VerifyingKey from pointing to the signature material
/// we get the ability to do `pubkey.validate(msg, signature)`
impl VerifyingKey for MultiEd25519PublicKey {
    type SigningKeyMaterial = MultiEd25519PrivateKey;
    type SignatureMaterial = MultiEd25519Signature;
}

impl fmt::Display for MultiEd25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()))
    }
}

impl fmt::Debug for MultiEd25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MultiEd25519PublicKey({})", self)
    }
}

impl Length for MultiEd25519PublicKey {
    fn length(&self) -> usize {
        self.public_keys.len() * ED25519_PUBLIC_KEY_LENGTH + 1
    }
}

impl ValidKey for MultiEd25519PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl MultiEd25519Signature {
    /// This method will also sort signatures based on index.
    pub fn new(
        signatures: Vec<(Ed25519Signature, u8)>,
    ) -> std::result::Result<Self, CryptoMaterialError> {
        let num_of_sigs = signatures.len();
        if num_of_sigs == 0 || num_of_sigs > MAX_NUM_OF_KEYS {
            return Err(CryptoMaterialError::ValidationError);
        }

        let mut sorted_signatures = signatures;
        sorted_signatures.sort_by(|a, b| a.1.cmp(&b.1));

        let mut sigvec = Vec::new();
        let mut bitvec = BitVec::from_elem(MAX_NUM_OF_KEYS, false);

        // Check if all indexes are unique and < MAX_NUM_OF_KEYS
        for (i, item) in sorted_signatures.into_iter().enumerate() {
            // If an index is out of range.
            if item.1 < MAX_NUM_OF_KEYS as u8 {
                // if an index has been set already (thus, there is a duplicate).
                if bitvec[i] {
                    return Err(CryptoMaterialError::BitVecError(
                        "Duplicate signature index".to_string(),
                    ));
                } else {
                    sigvec.push(item.0);
                    bitvec.set(item.1 as usize, true);
                }
            } else {
                return Err(CryptoMaterialError::BitVecError(
                    "Signature index is out of range".to_string(),
                ));
            }
        }
        Ok(MultiEd25519Signature {
            signatures: sigvec,
            bit_vector: bitvec,
        })
    }

    /// Getter signatures
    pub fn signatures(&self) -> &Vec<Ed25519Signature> {
        &self.signatures
    }

    /// Getter bit_vector
    pub fn bit_vector(&self) -> &BitVec {
        &self.bit_vector
    }

    /// Serialize a MultiEd25519Signature in the form of sig0||sig1||..sigN||bitvec.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self
            .signatures
            .iter()
            .flat_map(|sig| sig.to_bytes().to_vec())
            .collect();
        bytes.extend(self.bit_vector.to_bytes());
        bytes
    }
}

//////////////////////
// Signature Traits //
//////////////////////

impl TryFrom<&[u8]> for MultiEd25519Signature {
    type Error = CryptoMaterialError;

    /// Deserialize a MultiEd25519Signature. This method will also check for malleable signatures
    /// and bit_vector validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiEd25519Signature, CryptoMaterialError> {
        let length = bytes.len();
        let bitvec_num_of_bytes = length % ED25519_SIGNATURE_LENGTH;
        let num_of_sigs = length / ED25519_SIGNATURE_LENGTH;

        if num_of_sigs == 0
            || num_of_sigs > MAX_NUM_OF_KEYS
            || bitvec_num_of_bytes != BITVEC_NUM_OF_BYTES
        {
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let bit_vector = BitVec::from_bytes(&bytes[length - BITVEC_NUM_OF_BYTES..]);
        if count_set_bits(&bit_vector) != num_of_sigs {
            return Err(CryptoMaterialError::DeserializationError);
        }

        let signatures: Result<Vec<Ed25519Signature>, _> = bytes
            .chunks_exact(ED25519_SIGNATURE_LENGTH)
            .map(Ed25519Signature::try_from)
            .collect();
        signatures.map(|signatures| MultiEd25519Signature {
            signatures,
            bit_vector,
        })
    }
}

impl Length for MultiEd25519Signature {
    fn length(&self) -> usize {
        self.signatures.len() * ED25519_SIGNATURE_LENGTH + BITVEC_NUM_OF_BYTES
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for MultiEd25519Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

impl fmt::Display for MultiEd25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()[..]))
    }
}

impl fmt::Debug for MultiEd25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MultiEd25519Signature({})", self)
    }
}

impl ValidKey for MultiEd25519Signature {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Signature for MultiEd25519Signature {
    type VerifyingKeyMaterial = MultiEd25519PublicKey;
    type SigningKeyMaterial = MultiEd25519PrivateKey;

    /// Checks that `self` is valid for `message` using `public_key`.
    fn verify(&self, message: &HashValue, public_key: &MultiEd25519PublicKey) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in Move.
    fn verify_arbitrary_msg(
        &self,
        message: &[u8],
        public_key: &MultiEd25519PublicKey,
    ) -> Result<()> {
        if last_bit_set(&self.bit_vector) > public_key.length() {
            return Err(anyhow!(
                "{}",
                CryptoMaterialError::BitVecError("Signature index is out of range".to_string())
            ));
        }
        if count_set_bits(&self.bit_vector) < public_key.threshold as usize {
            return Err(anyhow!(
                "{}",
                CryptoMaterialError::BitVecError(
                    "Not enough signatures to meet the threshold".to_string()
                )
            ));
        }
        let mut bitvec_index = 0;
        // TODO use deterministic batch verification when gets available.
        for sig in &self.signatures {
            while !self.bit_vector[bitvec_index] {
                bitvec_index += 1;
            }
            sig.verify_arbitrary_msg(message, &public_key.public_keys[bitvec_index])?;
            bitvec_index += 1;
        }
        Ok(())
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl From<&Ed25519Signature> for MultiEd25519Signature {
    fn from(ed_signature: &Ed25519Signature) -> Self {
        MultiEd25519Signature {
            signatures: vec![ed_signature.clone()],
            bit_vector: BITVEC_FIRST_KEY_SET.clone(),
        }
    }
}

//////////////////////
// Helper functions //
//////////////////////

// Helper function required to MultiEd25519 keys to_bytes to add the threshold.
fn to_bytes<T: ValidKey>(keys: &[T], threshold: u8) -> Vec<u8> {
    let mut bytes: Vec<u8> = keys.iter().flat_map(ValidKey::to_bytes).collect();
    bytes.push(threshold);
    bytes
}

// Helper method to count bitvec's set bits in MultiEd25519 signatures.
fn count_set_bits(bitvec: &BitVec) -> usize {
    bitvec.iter().filter(|bit| *bit).count()
}

fn last_bit_set(bitvec: &BitVec) -> usize {
    let mut last_bit: usize = 0; // This is fine as we expect at least one set bit.
    for (i, bit) in bitvec.iter().enumerate() {
        if bit {
            last_bit = i;
        }
    }
    last_bit
}

// Helper method to get threshold from a serialized MultiEd25519 key payload.
fn check_and_get_threshold(
    bytes: &[u8],
    key_size: usize,
) -> std::result::Result<u8, CryptoMaterialError> {
    let payload_length = bytes.len();
    if bytes.is_empty() {
        return Err(CryptoMaterialError::WrongLengthError);
    }
    let threshold_num_of_bytes = payload_length % key_size;
    let num_of_keys = payload_length / key_size;
    let threshold_byte = bytes[bytes.len() - 1];

    if num_of_keys == 0 || num_of_keys > MAX_NUM_OF_KEYS || threshold_num_of_bytes != 1 {
        Err(CryptoMaterialError::WrongLengthError)
    } else if threshold_byte == 0 || threshold_byte > num_of_keys as u8 {
        Err(CryptoMaterialError::ValidationError)
    } else {
        Ok(threshold_byte)
    }
}
