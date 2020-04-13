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
use core::convert::TryFrom;
use libra_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use rand::Rng;
use std::{convert::TryInto, fmt};

const MAX_NUM_OF_KEYS: usize = 32;
const BITMAP_NUM_OF_BYTES: usize = 4;

/// Vector of private keys in the multi-key Ed25519 structure along with the threshold.
#[derive(DeserializeKey, Eq, PartialEq, SilentDisplay, SilentDebug, SerializeKey)]
pub struct SigningKeys {
    private_keys: Vec<Ed25519PrivateKey>,
    threshold: u8,
}

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(SigningKeys: Clone);

/// Vector of public keys in the multi-key Ed25519 structure along with the threshold.
#[derive(Clone, DeserializeKey, Eq, PartialEq, SerializeKey)]
pub struct VerifyingKeys {
    public_keys: Vec<Ed25519PublicKey>,
    threshold: u8,
}

/// Vector of the multi-key signatures along with a 32bit [u8; 4] bitmap required to map signatures
/// with their corresponding public keys.
///
/// Note that bits are read from left to right. For instance, in the following bitmap
/// [0b0001_0000, 0b0000_0000, 0b0000_0000, 0b0000_0001], the 3rd and 31st positions are set.
#[derive(Clone, DeserializeKey, Eq, PartialEq, SerializeKey)]
pub struct MultiSignature {
    signatures: Vec<Ed25519Signature>,
    bitmap: [u8; BITMAP_NUM_OF_BYTES],
}

impl SigningKeys {
    /// Construct a new multi_ed25519::SigningKeys.
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
            Ok(SigningKeys {
                private_keys,
                threshold,
            })
        }
    }

    /// Serialize a multi_ed25519::SigningKeys.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.private_keys, self.threshold)
    }
}

impl VerifyingKeys {
    /// Construct a new multi_ed25519::VerifyingKeys.
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
            Ok(VerifyingKeys {
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

    /// Serialize a multi_ed25519::VerifyingKeys.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.public_keys, self.threshold)
    }
}

///////////////////////
// SigningKeys Traits //
///////////////////////

/// Convenient method to create a multi_ed25519::SigningKeys from a single Ed25519PrivateKey.
impl From<&Ed25519PrivateKey> for SigningKeys {
    fn from(ed_private_key: &Ed25519PrivateKey) -> Self {
        SigningKeys {
            private_keys: vec![Ed25519PrivateKey::try_from(&ed_private_key.to_bytes()[..]).unwrap()],
            threshold: 1u8,
        }
    }
}

impl TPrivateKey for SigningKeys {
    type PublicKeyMaterial = VerifyingKeys;
}

impl TSigningKey for SigningKeys {
    type VerifyingKeyMaterial = VerifyingKeys;
    type SignatureMaterial = MultiSignature;

    // Sign a message with the minimum amount of keys to meet threshold (starting from left-most keys).
    fn sign_message(&self, message: &HashValue) -> MultiSignature {
        let mut signatures: Vec<Ed25519Signature> = Vec::with_capacity(self.threshold as usize);
        let mut bitmap = [0u8; BITMAP_NUM_OF_BYTES];
        signatures.extend(
            self.private_keys
                .iter()
                .take(self.threshold as usize)
                .enumerate()
                .map(|(i, item)| {
                    bitmap_set_bit(&mut bitmap, i);
                    item.sign_message(message)
                }),
        );
        MultiSignature { signatures, bitmap }
    }
}

// Generating a random K out-of N key for testing.
impl Uniform for SigningKeys {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let num_of_keys = rng.gen_range(1, MAX_NUM_OF_KEYS + 1);
        let mut private_keys: Vec<Ed25519PrivateKey> = Vec::with_capacity(num_of_keys);
        for _ in 0..num_of_keys {
            private_keys.push(
                Ed25519PrivateKey::try_from(
                    &ed25519_dalek::SecretKey::generate(rng).to_bytes()[..],
                )
                .unwrap(),
            );
        }
        let threshold = rng.gen_range(1, num_of_keys + 1) as u8;
        SigningKeys {
            private_keys,
            threshold,
        }
    }
}

impl TryFrom<&[u8]> for SigningKeys {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PrivateKey. This method will also check for key and threshold validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<SigningKeys, CryptoMaterialError> {
        if bytes.is_empty() {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let threshold = check_and_get_threshold(bytes, ED25519_PRIVATE_KEY_LENGTH)?;

        let private_keys: Result<Vec<Ed25519PrivateKey>, _> = bytes
            .chunks_exact(ED25519_PRIVATE_KEY_LENGTH)
            .map(Ed25519PrivateKey::try_from)
            .collect();

        private_keys.map(|private_keys| SigningKeys {
            private_keys,
            threshold,
        })
    }
}

impl Length for SigningKeys {
    fn length(&self) -> usize {
        self.private_keys.len() * ED25519_PRIVATE_KEY_LENGTH + 1
    }
}

impl ValidKey for SigningKeys {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Genesis for SigningKeys {
    fn genesis() -> Self {
        let mut buf = [0u8; ED25519_PRIVATE_KEY_LENGTH];
        buf[ED25519_PRIVATE_KEY_LENGTH - 1] = 1u8;
        SigningKeys {
            private_keys: vec![Ed25519PrivateKey::try_from(buf.as_ref()).unwrap()],
            threshold: 1u8,
        }
    }
}

//////////////////////
// VerifyingKeys Traits //
//////////////////////

/// Convenient method to create a MultiEd25519PublicKey from a single Ed25519PublicKey.
impl From<Ed25519PublicKey> for VerifyingKeys {
    fn from(ed_public_key: Ed25519PublicKey) -> Self {
        Self {
            public_keys: vec![ed_public_key],
            threshold: 1u8,
        }
    }
}

/// Implementing From<&SigningKeys<...>> allows to derive a public key in a more elegant fashion.
impl From<&SigningKeys> for VerifyingKeys {
    fn from(private_key: &SigningKeys) -> Self {
        let public_keys = private_key
            .private_keys
            .iter()
            .map(TPrivateKey::public_key)
            .collect();
        VerifyingKeys {
            public_keys,
            threshold: private_key.threshold,
        }
    }
}

/// We deduce VerifyingKeys from this.
impl TPublicKey for VerifyingKeys {
    type PrivateKeyMaterial = SigningKeys;
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for VerifyingKeys {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

impl TryFrom<&[u8]> for VerifyingKeys {
    type Error = CryptoMaterialError;

    /// Deserialize a multi_ed25519::VerifyingKeys. This method will also check for key and threshold
    /// validity, and will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<VerifyingKeys, CryptoMaterialError> {
        if bytes.is_empty() {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let threshold = check_and_get_threshold(bytes, ED25519_PUBLIC_KEY_LENGTH)?;
        let public_keys: Result<Vec<Ed25519PublicKey>, _> = bytes
            .chunks_exact(ED25519_PUBLIC_KEY_LENGTH)
            .map(Ed25519PublicKey::try_from)
            .collect();
        public_keys.map(|public_keys| VerifyingKeys {
            public_keys,
            threshold,
        })
    }
}

/// We deduce TVerifyingKey from pointing to the signature material
/// we get the ability to do `pubkey.validate(msg, signature)`
impl TVerifyingKey for VerifyingKeys {
    type SigningKeyMaterial = SigningKeys;
    type SignatureMaterial = MultiSignature;
}

impl fmt::Display for VerifyingKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()))
    }
}

impl fmt::Debug for VerifyingKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multi_ed25519::VerifyingKeys({})", self)
    }
}

impl Length for VerifyingKeys {
    fn length(&self) -> usize {
        self.public_keys.len() * ED25519_PUBLIC_KEY_LENGTH + 1
    }
}

impl ValidKey for VerifyingKeys {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl MultiSignature {
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

        let mut bitmap = [0u8; BITMAP_NUM_OF_BYTES];

        // Check if all indexes are unique and < MAX_NUM_OF_KEYS
        let (sigs, indexes): (Vec<_>, Vec<_>) = sorted_signatures.iter().cloned().unzip();
        for i in indexes {
            // If an index is out of range.
            if i < MAX_NUM_OF_KEYS as u8 {
                // if an index has been set already (thus, there is a duplicate).
                if bitmap_get_bit(bitmap, i as usize) {
                    return Err(CryptoMaterialError::BitVecError(
                        "Duplicate signature index".to_string(),
                    ));
                } else {
                    bitmap_set_bit(&mut bitmap, i as usize);
                }
            } else {
                return Err(CryptoMaterialError::BitVecError(
                    "Signature index is out of range".to_string(),
                ));
            }
        }
        Ok(MultiSignature {
            signatures: sigs,
            bitmap,
        })
    }

    /// Getter signatures.
    pub fn signatures(&self) -> &Vec<Ed25519Signature> {
        &self.signatures
    }

    /// Getter bitmap.
    pub fn bitmap(&self) -> &[u8; BITMAP_NUM_OF_BYTES] {
        &self.bitmap
    }

    /// Serialize a multi_ed25519::MultiSignature in the form of sig0||sig1||..sigN||bitmap.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self
            .signatures
            .iter()
            .flat_map(|sig| sig.to_bytes().to_vec())
            .collect();
        bytes.extend(&self.bitmap[..]);
        bytes
    }
}

//////////////////////
// MultiSignature Traits //
//////////////////////

impl TryFrom<&[u8]> for MultiSignature {
    type Error = CryptoMaterialError;

    /// Deserialize a multi_ed25519::MultiSignature. This method will also check for malleable signatures
    /// and bitmap validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<MultiSignature, CryptoMaterialError> {
        let length = bytes.len();
        let bitmap_num_of_bytes = length % ED25519_SIGNATURE_LENGTH;
        let num_of_sigs = length / ED25519_SIGNATURE_LENGTH;

        if num_of_sigs == 0
            || num_of_sigs > MAX_NUM_OF_KEYS
            || bitmap_num_of_bytes != BITMAP_NUM_OF_BYTES
        {
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let bitmap = bytes[length - BITMAP_NUM_OF_BYTES..].try_into().unwrap();
        if bitmap_count_ones(bitmap) != num_of_sigs as u32 {
            return Err(CryptoMaterialError::DeserializationError);
        }

        let signatures: Result<Vec<Ed25519Signature>, _> = bytes
            .chunks_exact(ED25519_SIGNATURE_LENGTH)
            .map(Ed25519Signature::try_from)
            .collect();
        signatures.map(|signatures| MultiSignature { signatures, bitmap })
    }
}

impl Length for MultiSignature {
    fn length(&self) -> usize {
        self.signatures.len() * ED25519_SIGNATURE_LENGTH + BITMAP_NUM_OF_BYTES
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for MultiSignature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

impl fmt::Display for MultiSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()[..]))
    }
}

impl fmt::Debug for MultiSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multi_ed25519::MultiSignature({})", self)
    }
}

impl ValidKey for MultiSignature {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl TSignature for MultiSignature {
    type VerifyingKeyMaterial = VerifyingKeys;
    type SigningKeyMaterial = SigningKeys;

    /// Checks that `self` is valid for `message` using `public_key`.
    fn verify(&self, message: &HashValue, public_key: &VerifyingKeys) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in Move.
    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &VerifyingKeys) -> Result<()> {
        let last_bit = bitmap_last_set_bit(self.bitmap);
        if last_bit == None || last_bit.unwrap() as usize > public_key.length() {
            return Err(anyhow!(
                "{}",
                CryptoMaterialError::BitVecError("Signature index is out of range".to_string())
            ));
        }
        if bitmap_count_ones(self.bitmap) < public_key.threshold as u32 {
            return Err(anyhow!(
                "{}",
                CryptoMaterialError::BitVecError(
                    "Not enough signatures to meet the threshold".to_string()
                )
            ));
        }
        let mut bitmap_index = 0;
        // TODO use deterministic batch verification when gets available.
        for sig in &self.signatures {
            while !bitmap_get_bit(self.bitmap, bitmap_index) {
                bitmap_index += 1;
            }
            sig.verify_arbitrary_msg(message, &public_key.public_keys[bitmap_index as usize])?;
            bitmap_index += 1;
        }
        Ok(())
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl From<Ed25519Signature> for MultiSignature {
    fn from(ed_signature: Ed25519Signature) -> Self {
        Self {
            signatures: vec![ed_signature],
            // "1000_0000 0000_0000 0000_0000 0000_0000"
            bitmap: [0b1000_0000u8, 0u8, 0u8, 0u8],
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

fn bitmap_set_bit(input: &mut [u8; BITMAP_NUM_OF_BYTES], index: usize) {
    let bucket = index / 8;
    // It's always invoked with index < 32, thus there is no need to check range.
    let bucket_pos = index - (bucket * 8);
    input[bucket] |= 128 >> bucket_pos as u8;
}

// Helper method to get the input's bit at index.
fn bitmap_get_bit(input: [u8; BITMAP_NUM_OF_BYTES], index: usize) -> bool {
    let bucket = index / 8;
    // It's always invoked with index < 32, thus there is no need to check range.
    let bucket_pos = index - (bucket * 8);
    (input[bucket] & (128 >> bucket_pos as u8)) != 0
}

// Returns the number of set bits.
fn bitmap_count_ones(input: [u8; BITMAP_NUM_OF_BYTES]) -> u32 {
    input.iter().map(|a| a.count_ones()).sum()
}

// Find the last set bit.
fn bitmap_last_set_bit(input: [u8; BITMAP_NUM_OF_BYTES]) -> Option<u8> {
    input
        .iter()
        .rev()
        .enumerate()
        .find(|(_, byte)| byte != &&0u8)
        .map(|(i, byte)| (8 * (BITMAP_NUM_OF_BYTES - i) - byte.trailing_zeros() as usize - 1) as u8)
}

#[test]
fn bitmap_tests() {
    let mut bitmap = [0b0100_0000u8, 0b1111_1111u8, 0u8, 0b1000_0000u8];
    assert!(!bitmap_get_bit(bitmap, 0));
    assert!(bitmap_get_bit(bitmap, 1));
    for i in 8..16 {
        assert!(bitmap_get_bit(bitmap, i));
    }
    for i in 16..24 {
        assert!(!bitmap_get_bit(bitmap, i));
    }
    assert!(bitmap_get_bit(bitmap, 24));
    assert!(!bitmap_get_bit(bitmap, 31));
    assert_eq!(bitmap_last_set_bit(bitmap), Some(24));

    bitmap_set_bit(&mut bitmap, 30);
    assert!(bitmap_get_bit(bitmap, 30));
    assert_eq!(bitmap_last_set_bit(bitmap), Some(30));
}
