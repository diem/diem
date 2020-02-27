// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an API for the accountable threshold multi-sig PureEdDSA signature scheme
//! over the ed25519 twisted Edwards curve as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
//!
//! Signature verification also checks and rejects non-canonical signatures.

use crate::ed25519::{
    Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PRIVATE_KEY_LENGTH,
    ED25519_PUBLIC_KEY_LENGTH, ED25519_SIGNATURE_LENGTH,
};
use crate::{traits::*, HashValue};
use anyhow::{anyhow, Result};
use bit_vec::BitVec;
use core::convert::TryFrom;
use libra_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use once_cell::sync::Lazy;
use std::fmt;

const MAX_NUM_OF_KEYS: usize = 32;
const BITVEC_NUM_OF_BYTES: usize = MAX_NUM_OF_KEYS / 8;
static BITVEC_FIRST_KEY_SET: Lazy<BitVec> =
    Lazy::new(|| BitVec::from_bytes(&[0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]));

/// Vector of private keys in the multi-key Ed25519 structure along with the threshold.
#[derive(DeserializeKey, SilentDisplay, SilentDebug, SerializeKey)]
pub struct MultiEd25519PrivateKey(Vec<Ed25519PrivateKey>, u8);

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(MultiEd25519PrivateKey: Clone);

/// Vector of public keys in the multi-key Ed25519 structure along with the threshold.
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct MultiEd25519PublicKey(pub(crate) Vec<Ed25519PublicKey>, pub(crate) u8);

/// Vector of the multi-key signatures along with their index required to map signatures with
/// their corresponding public keys.
#[derive(DeserializeKey, Clone, SerializeKey)]
pub struct MultiEd25519Signature(pub(crate) Vec<Ed25519Signature>, pub(crate) BitVec);

impl MultiEd25519PrivateKey {
    /// Construct a new MultiEd25519PrivateKey.
    pub fn new(
        private_keys: Vec<Ed25519PrivateKey>,
        threshold: u8,
    ) -> std::result::Result<Self, CryptoMaterialError> {
        let num_of_keys = private_keys.len();
        if threshold != 0 && num_of_keys >= threshold as usize && num_of_keys <= MAX_NUM_OF_KEYS {
            Ok(MultiEd25519PrivateKey(private_keys, threshold))
        } else {
            Err(CryptoMaterialError::WrongLengthError)
        }
    }

    /// Serialize a MultiEd25519PrivateKey.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.0, self.1)
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
        if threshold != 0 && num_of_keys >= threshold as usize && num_of_keys <= MAX_NUM_OF_KEYS {
            Ok(MultiEd25519PublicKey(public_keys, threshold))
        } else {
            Err(CryptoMaterialError::WrongLengthError)
        }
    }

    /// Serialize a MultiEd25519PublicKey.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.0, self.1)
    }
}

///////////////////////
// PrivateKey Traits //
///////////////////////

impl PrivateKey for MultiEd25519PrivateKey {
    type PublicKeyMaterial = MultiEd25519PublicKey;
}

impl SigningKey for MultiEd25519PrivateKey {
    type VerifyingKeyMaterial = MultiEd25519PublicKey;
    type SignatureMaterial = MultiEd25519Signature;

    // Sign a message with the minimum amount of keys to meet threshold (starting from left-most keys).
    fn sign_message(&self, message: &HashValue) -> MultiEd25519Signature {
        let mut ed25519_signatures: Vec<Ed25519Signature> = Vec::with_capacity(self.1 as usize);
        let mut bitvec = BitVec::from_elem(MAX_NUM_OF_KEYS, false);

        for (i, item) in self.0.iter().enumerate() {
            if i < self.1 as usize {
                ed25519_signatures.push(item.sign_message(message));
                bitvec.set(i, true);
            } else {
                break;
            }
        }
        MultiEd25519Signature(ed25519_signatures, bitvec)
    }
}

// Generating a random 2 out-of 3 key for testing.
impl Uniform for MultiEd25519PrivateKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let mut private_keys: Vec<Ed25519PrivateKey> = Vec::with_capacity(3);
        for _ in 0..3 {
            private_keys.push(
                Ed25519PrivateKey::try_from(
                    &ed25519_dalek::SecretKey::generate(rng).to_bytes()[..],
                )
                .unwrap(),
            );
        }
        MultiEd25519PrivateKey(private_keys, 2)
    }
}

impl PartialEq<Self> for MultiEd25519PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for MultiEd25519PrivateKey {}

// We could have a distinct kind of validation for the PrivateKey, for
// ex. checking the derived PublicKey is valid?
impl TryFrom<&[u8]> for MultiEd25519PrivateKey {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PrivateKey. This method will also check for key and threshold validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiEd25519PrivateKey, CryptoMaterialError> {
        let payload_length = bytes.len();
        // Special case for single keys to speed up deserialization.
        if payload_length == ED25519_PRIVATE_KEY_LENGTH {
            return Ed25519PrivateKey::try_from(bytes)
                .map(|private_key| MultiEd25519PrivateKey(vec![private_key], 1u8));
        }

        let threshold = get_threshold(
            payload_length,
            ED25519_PRIVATE_KEY_LENGTH,
            bytes[payload_length - 1],
        )?;

        let mut private_keys: Vec<Ed25519PrivateKey> = vec![];
        let multi_key = bytes
            .chunks(ED25519_PRIVATE_KEY_LENGTH)
            .filter(|chunk| chunk.len() == ED25519_PRIVATE_KEY_LENGTH)
            .try_for_each(|key_bytes| deserialize_and_push(key_bytes, &mut private_keys));

        multi_key.map(|_| MultiEd25519PrivateKey(private_keys, threshold))
    }
}

impl Length for MultiEd25519PrivateKey {
    fn length(&self) -> usize {
        self.to_bytes().len()
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
        buf[ED25519_PRIVATE_KEY_LENGTH - 1] = 1;
        MultiEd25519PrivateKey(vec![Ed25519PrivateKey::try_from(buf.as_ref()).unwrap()], 1)
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////

/// Convenient method to create a MultiEd25519PublicKey from a single Ed25519PublicKey.
impl From<&Ed25519PublicKey> for MultiEd25519PublicKey {
    fn from(ed_public_key: &Ed25519PublicKey) -> Self {
        MultiEd25519PublicKey(vec![ed_public_key.clone()], 1u8)
    }
}

/// Implementing From<&PrivateKey<...>> allows to derive a public key in a more elegant fashion.
impl From<&MultiEd25519PrivateKey> for MultiEd25519PublicKey {
    fn from(private_key: &MultiEd25519PrivateKey) -> Self {
        let public_key = private_key
            .0
            .iter()
            .map(|ed_private_key| ed_private_key.public_key())
            .collect();
        MultiEd25519PublicKey(public_key, private_key.1)
    }
}

/// We deduce PublicKey from this.
impl PublicKey for MultiEd25519PublicKey {
    type PrivateKeyMaterial = MultiEd25519PrivateKey;
}

/// Get the hash output of the serialized MultiEd25519PublicKey.
impl std::hash::Hash for MultiEd25519PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

// Those are required by the implementation of hash above.
impl PartialEq for MultiEd25519PublicKey {
    fn eq(&self, other: &MultiEd25519PublicKey) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for MultiEd25519PublicKey {}

impl TryFrom<&[u8]> for MultiEd25519PublicKey {
    type Error = CryptoMaterialError;

    /// Deserialize a MultiEd25519PublicKey. This method will also check for key and threshold
    /// validity, and will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiEd25519PublicKey, CryptoMaterialError> {
        let payload_length = bytes.len();
        // Special case for single keys to speed up deserialization.
        if payload_length == ED25519_PUBLIC_KEY_LENGTH {
            return Ed25519PublicKey::try_from(bytes)
                .map(|public_key| MultiEd25519PublicKey(vec![public_key], 1u8));
        }
        let threshold = get_threshold(
            payload_length,
            ED25519_PRIVATE_KEY_LENGTH,
            bytes[payload_length - 1],
        )?;

        let mut public_keys: Vec<Ed25519PublicKey> = vec![];
        let multi_key = bytes
            .chunks(ED25519_PUBLIC_KEY_LENGTH)
            .filter(|chunk| chunk.len() == ED25519_PUBLIC_KEY_LENGTH)
            .try_for_each(|key_bytes| deserialize_and_push(key_bytes, &mut public_keys));

        multi_key.map(|_| MultiEd25519PublicKey(public_keys, threshold))
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
        self.to_bytes().len()
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
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let mut sorted_signatures = signatures;
        sorted_signatures.sort_by(|a, b| a.1.cmp(&b.1));

        let mut sigvec: Vec<Ed25519Signature> = Vec::with_capacity(num_of_sigs);
        let mut bitvec = BitVec::from_elem(MAX_NUM_OF_KEYS, false);

        // Check if all indexes are unique and < MAX_NUM_OF_KEYS
        for (i, item) in sorted_signatures.iter().enumerate() {
            // If an index is out of range.
            if item.1 < MAX_NUM_OF_KEYS as u8 {
                // if an index has been set already (thus, there is a duplicate).
                if bitvec[i] {
                    return Err(CryptoMaterialError::BitVecError);
                } else {
                    sigvec[i] = item.0.clone();
                    bitvec.set(i, true);
                }
            } else {
                return Err(CryptoMaterialError::BitVecError);
            }
        }
        Ok(MultiEd25519Signature(sigvec, bitvec))
    }

    /// Serialize a MultiEd25519Signature in the form of sig0||sig1||..sigN||bitvec.
    /// Note that we omit bitvec if all bits from index = zero to index = signatures.len() - 1
    /// are set.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        self.0.iter().for_each(|signature| {
            bytes.extend(signature.to_bytes().to_vec());
        });
        // TODO compress even more, i.e., we don't always need up to 4bytes for bitvec.
        // No need to attach the bitvector when there is no gap between the indexes.
        if (0..self.0.len()).any(|i| !self.1[i]) {
            bytes.extend(self.1.to_bytes());
        }
        bytes
    }
}

impl TryFrom<&[u8]> for MultiEd25519Signature {
    type Error = CryptoMaterialError;

    /// Deserialize a MultiEd25519Signature. This method will also check for bitvec validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiEd25519Signature, CryptoMaterialError> {
        let length = bytes.len();
        // Special case for single signatures to speed up deserialization.
        if length == ED25519_SIGNATURE_LENGTH {
            return Ed25519Signature::try_from(bytes).map(|signature| {
                MultiEd25519Signature(vec![signature], BITVEC_FIRST_KEY_SET.clone())
            });
        }
        let bitvec_num_of_bytes = length % ED25519_SIGNATURE_LENGTH;
        let num_of_sigs = length / ED25519_SIGNATURE_LENGTH;

        if num_of_sigs == 0
            || num_of_sigs > MAX_NUM_OF_KEYS
            || (bitvec_num_of_bytes != 0 && bitvec_num_of_bytes != BITVEC_NUM_OF_BYTES)
        {
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let bitvec: BitVec = if bitvec_num_of_bytes == BITVEC_NUM_OF_BYTES {
            let temp = BitVec::from_bytes(&bytes[length - 4..length]);
            let count_set_bits = count_set_bits(&temp);
            if count_set_bits != num_of_sigs || count_set_bits != 0 {
                return Err(CryptoMaterialError::DeserializationError);
            }
            temp
        } else {
            let mut temp = BitVec::from_elem(MAX_NUM_OF_KEYS, false);
            for i in 0..num_of_sigs {
                temp.set(i, true);
            }
            temp
        };

        let mut signatures: Vec<Ed25519Signature> = vec![];
        let multi_signature = bytes
            .chunks(ED25519_SIGNATURE_LENGTH)
            .filter(|chunk| chunk.len() == ED25519_SIGNATURE_LENGTH)
            .try_for_each(|sig_bytes| deserialize_and_push(sig_bytes, &mut signatures));

        multi_signature.map(|_| MultiEd25519Signature(signatures, bitvec))
    }
}

impl Length for MultiEd25519Signature {
    fn length(&self) -> usize {
        self.to_bytes().len()
    }
}

impl std::hash::Hash for MultiEd25519Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

// Those are required by the implementation of hash above.
impl PartialEq for MultiEd25519Signature {
    fn eq(&self, other: &MultiEd25519Signature) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Eq for MultiEd25519Signature {}

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

//////////////////////
// Signature Traits //
//////////////////////

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
        if last_bit_set(&self.1) > public_key.length()
            || count_set_bits(&self.1) < public_key.1 as usize
        {
            return Err(anyhow!("{}", CryptoMaterialError::ValidationError));
        }
        let mut bitvec_index = 0;
        // TODO use deterministic batch verification
        for sig in &self.0 {
            while !self.1[bitvec_index] {
                bitvec_index += 1;
            }
            sig.verify_arbitrary_msg(message, &public_key.0[bitvec_index])?;
            bitvec_index += 1;
        }
        Ok(())
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

//////////////////////
// Helper functions //
//////////////////////

// Helper function required to MultiEd25519 keys to_bytes to add or omit the threshold.
fn to_bytes<T: ValidKey>(keys: &[T], threshold: u8) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    keys.iter().for_each(|key| {
        bytes.extend(key.to_bytes());
    });
    // No need to attach the threshold in N out-of N.
    if keys.len() > threshold as usize {
        bytes.push(threshold);
    }
    bytes
}

// Helper method to count bitvec's set bits in MultiEd25519 signatures.
fn count_set_bits(bitvec: &BitVec) -> usize {
    let mut count: usize = 0;
    for bit in bitvec.iter() {
        if bit {
            count += 1;
        }
    }
    count
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
fn get_threshold(
    payload_length: usize,
    key_size: usize,
    byte: u8,
) -> std::result::Result<u8, CryptoMaterialError> {
    let threshold_num_of_bytes = payload_length % key_size;
    let num_of_keys = payload_length / key_size;

    if num_of_keys == 0 || num_of_keys > MAX_NUM_OF_KEYS || threshold_num_of_bytes > 1 {
        return Err(CryptoMaterialError::WrongLengthError);
    }

    if threshold_num_of_bytes == 1 {
        // We don't attach the threshold on N out-of N as well.
        if byte >= num_of_keys as u8 {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        Ok(byte)
    } else {
        Ok(num_of_keys as u8)
    }
}

// Helper function required to MultiEd25519 keys and signature try_from.
fn deserialize_and_push<'a, T: TryFrom<&'a [u8], Error = CryptoMaterialError>>(
    bytes: &'a [u8],
    keys: &mut Vec<T>,
) -> std::result::Result<(), CryptoMaterialError> {
    T::try_from(bytes).map(|key| {
        keys.push(key);
    })
}
