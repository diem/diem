// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an API for the accountable threshold multi-sig PureEdDSA signature scheme
//! over the ed25519 twisted Edwards curve as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
//!
//! Signature verification also checks and rejects non-canonical signatures.

use crate::{ed25519, traits::*, HashValue};
use anyhow::{anyhow, Result};
use core::convert::TryFrom;
use libra_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
use rand::Rng;
use std::{convert::TryInto, fmt};

const MAX_NUM_OF_KEYS: usize = 32;
const BITMAP_NUM_OF_BYTES: usize = 4;

/// Vector of private keys in the multi-key Ed25519 structure along with the threshold.
#[derive(DeserializeKey, Eq, PartialEq, SilentDisplay, SilentDebug, SerializeKey)]
pub struct PrivateKey {
    private_keys: Vec<ed25519::PrivateKey>,
    threshold: u8,
}

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(PrivateKey: Clone);

/// Vector of public keys in the multi-key Ed25519 structure along with the threshold.
#[derive(Clone, DeserializeKey, Eq, PartialEq, SerializeKey)]
pub struct PublicKey {
    public_keys: Vec<ed25519::PublicKey>,
    threshold: u8,
}

/// Vector of the multi-key signatures along with a 32bit [u8; 4] bitmap required to map signatures
/// with their corresponding public keys.
///
/// Note that bits are read from left to right. For instance, in the following bitmap
/// [0b0001_0000, 0b0000_0000, 0b0000_0000, 0b0000_0001], the 3rd and 31st positions are set.
#[derive(Clone, DeserializeKey, Eq, PartialEq, SerializeKey)]
pub struct Signature {
    signatures: Vec<ed25519::Signature>,
    bitmap: [u8; BITMAP_NUM_OF_BYTES],
}

impl PrivateKey {
    /// Construct a new multi_ed25519::PrivateKey.
    pub fn new(
        private_keys: Vec<ed25519::PrivateKey>,
        threshold: u8,
    ) -> std::result::Result<Self, CryptoMaterialError> {
        let num_of_keys = private_keys.len();
        if threshold == 0 || num_of_keys < threshold as usize {
            Err(CryptoMaterialError::ValidationError)
        } else if num_of_keys > MAX_NUM_OF_KEYS {
            Err(CryptoMaterialError::WrongLengthError)
        } else {
            Ok(PrivateKey {
                private_keys,
                threshold,
            })
        }
    }

    /// Serialize a multi_ed25519::PrivateKey.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.private_keys, self.threshold)
    }
}

impl PublicKey {
    /// Construct a new multi_ed25519::PublicKey.
    /// --- Rules ---
    /// a) threshold cannot be zero.
    /// b) public_keys.len() should be equal to or larger than threshold.
    /// c) support up to MAX_NUM_OF_KEYS public keys.
    pub fn new(
        public_keys: Vec<ed25519::PublicKey>,
        threshold: u8,
    ) -> std::result::Result<Self, CryptoMaterialError> {
        let num_of_keys = public_keys.len();
        if threshold == 0 || num_of_keys < threshold as usize {
            Err(CryptoMaterialError::ValidationError)
        } else if num_of_keys > MAX_NUM_OF_KEYS {
            Err(CryptoMaterialError::WrongLengthError)
        } else {
            Ok(PublicKey {
                public_keys,
                threshold,
            })
        }
    }

    /// Getter public_keys
    pub fn public_keys(&self) -> &Vec<ed25519::PublicKey> {
        &self.public_keys
    }

    /// Getter threshold
    pub fn threshold(&self) -> &u8 {
        &self.threshold
    }

    /// Serialize a multi_ed25519::PublicKey.
    pub fn to_bytes(&self) -> Vec<u8> {
        to_bytes(&self.public_keys, self.threshold)
    }
}

///////////////////////
// PrivateKey Traits //
///////////////////////

/// Convenient method to create a multi_ed25519::PrivateKey from a single ed25519::PrivateKey.
impl From<&ed25519::PrivateKey> for PrivateKey {
    fn from(ed_private_key: &ed25519::PrivateKey) -> Self {
        PrivateKey {
            private_keys: vec![
                ed25519::PrivateKey::try_from(&ed_private_key.to_bytes()[..]).unwrap(),
            ],
            threshold: 1u8,
        }
    }
}

impl PrivateKeyExt for PrivateKey {
    type PublicKeyMaterial = PublicKey;
}

impl SigningKey for PrivateKey {
    type VerifyingKeyMaterial = PublicKey;
    type SignatureMaterial = Signature;

    // Sign a message with the minimum amount of keys to meet threshold (starting from left-most keys).
    fn sign_message(&self, message: &HashValue) -> Signature {
        let mut signatures: Vec<ed25519::Signature> = Vec::with_capacity(self.threshold as usize);
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
        Signature { signatures, bitmap }
    }
}

// Generating a random K out-of N key for testing.
impl Uniform for PrivateKey {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let num_of_keys = rng.gen_range(1, MAX_NUM_OF_KEYS + 1);
        let mut private_keys: Vec<ed25519::PrivateKey> = Vec::with_capacity(num_of_keys);
        for _ in 0..num_of_keys {
            private_keys.push(
                ed25519::PrivateKey::try_from(
                    &ed25519_dalek::SecretKey::generate(rng).to_bytes()[..],
                )
                .unwrap(),
            );
        }
        let threshold = rng.gen_range(1, num_of_keys + 1) as u8;
        PrivateKey {
            private_keys,
            threshold,
        }
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = CryptoMaterialError;

    /// Deserialize an ed25519::PrivateKey. This method will also check for key and threshold validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<PrivateKey, CryptoMaterialError> {
        if bytes.is_empty() {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let threshold = check_and_get_threshold(bytes, ed25519::PRIVATE_KEY_LENGTH)?;

        let private_keys: Result<Vec<ed25519::PrivateKey>, _> = bytes
            .chunks_exact(ed25519::PRIVATE_KEY_LENGTH)
            .map(ed25519::PrivateKey::try_from)
            .collect();

        private_keys.map(|private_keys| PrivateKey {
            private_keys,
            threshold,
        })
    }
}

impl Length for PrivateKey {
    fn length(&self) -> usize {
        self.private_keys.len() * ed25519::PRIVATE_KEY_LENGTH + 1
    }
}

impl ValidKey for PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Genesis for PrivateKey {
    fn genesis() -> Self {
        let mut buf = [0u8; ed25519::PRIVATE_KEY_LENGTH];
        buf[ed25519::PRIVATE_KEY_LENGTH - 1] = 1u8;
        PrivateKey {
            private_keys: vec![ed25519::PrivateKey::try_from(buf.as_ref()).unwrap()],
            threshold: 1u8,
        }
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////

/// Convenient method to create a MultiEd25519PublicKey from a single Ed25519PublicKey.
impl From<ed25519::PublicKey> for PublicKey {
    fn from(ed_public_key: ed25519::PublicKey) -> Self {
        Self {
            public_keys: vec![ed_public_key],
            threshold: 1u8,
        }
    }
}

/// Implementing From<&PrivateKey<...>> allows to derive a public key in a more elegant fashion.
impl From<&PrivateKey> for PublicKey {
    fn from(private_key: &PrivateKey) -> Self {
        let public_keys = private_key
            .private_keys
            .iter()
            .map(PrivateKeyExt::public_key)
            .collect();
        PublicKey {
            public_keys,
            threshold: private_key.threshold,
        }
    }
}

/// We deduce PublicKey from this.
impl PublicKeyExt for PublicKey {
    type PrivateKeyMaterial = PrivateKey;
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_pubkey = self.to_bytes();
        state.write(&encoded_pubkey);
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = CryptoMaterialError;

    /// Deserialize a multi_ed25519::PublicKey. This method will also check for key and threshold
    /// validity, and will only deserialize keys that are safe against small subgroup attacks.
    fn try_from(bytes: &[u8]) -> std::result::Result<PublicKey, CryptoMaterialError> {
        if bytes.is_empty() {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let threshold = check_and_get_threshold(bytes, ed25519::PUBLIC_KEY_LENGTH)?;
        let public_keys: Result<Vec<ed25519::PublicKey>, _> = bytes
            .chunks_exact(ed25519::PUBLIC_KEY_LENGTH)
            .map(ed25519::PublicKey::try_from)
            .collect();
        public_keys.map(|public_keys| PublicKey {
            public_keys,
            threshold,
        })
    }
}

/// We deduce VerifyingKey from pointing to the signature material
/// we get the ability to do `pubkey.validate(msg, signature)`
impl VerifyingKey for PublicKey {
    type SigningKeyMaterial = PrivateKey;
    type SignatureMaterial = Signature;
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multi_ed25519::PublicKey({})", self)
    }
}

impl Length for PublicKey {
    fn length(&self) -> usize {
        self.public_keys.len() * ed25519::PUBLIC_KEY_LENGTH + 1
    }
}

impl ValidKey for PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Signature {
    /// This method will also sort signatures based on index.
    pub fn new(
        signatures: Vec<(ed25519::Signature, u8)>,
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
        Ok(Signature {
            signatures: sigs,
            bitmap,
        })
    }

    /// Getter signatures.
    pub fn signatures(&self) -> &Vec<ed25519::Signature> {
        &self.signatures
    }

    /// Getter bitmap.
    pub fn bitmap(&self) -> &[u8; BITMAP_NUM_OF_BYTES] {
        &self.bitmap
    }

    /// Serialize a multi_ed25519::Signature in the form of sig0||sig1||..sigN||bitmap.
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
// Signature Traits //
//////////////////////

impl TryFrom<&[u8]> for Signature {
    type Error = CryptoMaterialError;

    /// Deserialize a multi_ed25519::Signature. This method will also check for malleable signatures
    /// and bitmap validity.
    fn try_from(bytes: &[u8]) -> std::result::Result<Signature, CryptoMaterialError> {
        let length = bytes.len();
        let bitmap_num_of_bytes = length % ed25519::SIGNATURE_LENGTH;
        let num_of_sigs = length / ed25519::SIGNATURE_LENGTH;

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

        let signatures: Result<Vec<ed25519::Signature>, _> = bytes
            .chunks_exact(ed25519::SIGNATURE_LENGTH)
            .map(ed25519::Signature::try_from)
            .collect();
        signatures.map(|signatures| Signature { signatures, bitmap })
    }
}

impl Length for Signature {
    fn length(&self) -> usize {
        self.signatures.len() * ed25519::SIGNATURE_LENGTH + BITMAP_NUM_OF_BYTES
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()[..]))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multi_ed25519::Signature({})", self)
    }
}

impl ValidKey for Signature {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl SignatureExt for Signature {
    type VerifyingKeyMaterial = PublicKey;
    type SigningKeyMaterial = PrivateKey;

    /// Checks that `self` is valid for `message` using `public_key`.
    fn verify(&self, message: &HashValue, public_key: &PublicKey) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in Move.
    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &PublicKey) -> Result<()> {
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

impl From<ed25519::Signature> for Signature {
    fn from(ed_signature: ed25519::Signature) -> Self {
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
