// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dispatch::{CostedReturnType, NativeReturnType, Result, StackAccessor};
use bit_vec::BitVec;
use failure::prelude::*;
use nextgen_crypto::{
    ed25519::{self, Ed25519PublicKey, Ed25519Signature},
    traits::*,
    HashValue,
};
use std::convert::TryFrom;
use types::byte_array::ByteArray;

// TODO: Talk to Crypto to determine these costs
const ED25519_COST: u64 = 35;
const BATCH_ED25519_COST: u64 = 30;

const BITMAP_SIZE: usize = 32;

pub fn native_ed25519_signature_verification<T: StackAccessor>(
    mut accessor: T,
) -> Result<CostedReturnType> {
    let msg = accessor.get_byte_array()?;
    let pubkey = accessor.get_byte_array()?;
    let signature = accessor.get_byte_array()?;

    let native_cost = ED25519_COST * msg.len() as u64;

    let sig = ed25519::Ed25519Signature::try_from(signature.as_bytes())?;
    let pk = ed25519::Ed25519PublicKey::try_from(pubkey.as_bytes())?;

    match sig.verify_arbitrary_msg(msg.as_bytes(), &pk) {
        Ok(()) => Ok(CostedReturnType::new(
            native_cost,
            NativeReturnType::Bool(true),
        )),
        Err(_) => Ok(CostedReturnType::new(
            native_cost,
            NativeReturnType::Bool(false),
        )),
    }
}

/// Batch verify a collection of signatures using a bitmap for matching signatures to keys.
pub fn native_ed25519_threshold_signature_verification<T: StackAccessor>(
    mut accessor: T,
) -> Result<CostedReturnType> {
    let message = accessor.get_byte_array()?;
    let public_keys = accessor.get_byte_array()?;
    let signatures = accessor.get_byte_array()?;
    let bitmap = accessor.get_byte_array()?;

    let num_of_sigs =
        ed25519_threshold_signature_verification(&bitmap, &signatures, &public_keys, &message);

    match num_of_sigs {
        Ok(num_of_sigs) => {
            let native_cost = BATCH_ED25519_COST * num_of_sigs * message.len() as u64;
            Ok(CostedReturnType::new(
                native_cost,
                NativeReturnType::U64(num_of_sigs),
            ))
        }
        Err(err) => Err(err),
    }
}

fn ed25519_threshold_signature_verification(
    bitmap: &ByteArray,
    signatures: &ByteArray,
    public_keys: &ByteArray,
    message: &ByteArray,
) -> Result<u64> {
    let bitvec = BitVec::from_bytes(bitmap.as_bytes());

    let num_of_sigs = sanity_check(&bitvec, &signatures, &public_keys).unwrap();

    let sig_chunks: ::std::result::Result<Vec<_>, _> = signatures
        .as_bytes()
        .chunks(64)
        .map(Ed25519Signature::try_from)
        .collect();

    match sig_chunks {
        Ok(signatures) => {
            let key_chunks: ::std::result::Result<Vec<_>, _> = public_keys
                .as_bytes()
                .chunks(32)
                .map(Ed25519PublicKey::try_from)
                .collect();

            match key_chunks {
                Ok(keys) => {
                    let keys_and_signatures =
                        matching_keys_and_signatures(num_of_sigs, bitvec, signatures, keys);

                    match Ed25519Signature::batch_verify_signatures(
                        &(HashValue::from_slice(message.as_bytes()).unwrap()),
                        keys_and_signatures,
                    ) {
                        Ok(()) => Ok(num_of_sigs),
                        Err(_) => bail!("Batch verification failed"),
                    }
                }
                Err(_) => bail!("Key deserialization error"),
            }
        }
        Err(_) => bail!("Signature deserialization error"),
    }
}

fn matching_keys_and_signatures(
    num_of_sigs: u64,
    bitmap: BitVec,
    signatures: Vec<Ed25519Signature>,
    public_keys: Vec<Ed25519PublicKey>,
) -> Vec<(Ed25519PublicKey, Ed25519Signature)> {
    let mut sig_index = 0;
    let mut keys_and_signatures: Vec<(Ed25519PublicKey, Ed25519Signature)> =
        Vec::with_capacity(num_of_sigs as usize);
    for (key_index, bit) in bitmap.iter().enumerate() {
        if bit {
            keys_and_signatures.push((
                // unwrap() will always succeed because we already did the sanity check.
                public_keys.get(key_index).unwrap().clone(),
                signatures.get(sig_index).unwrap().clone(),
            ));
            sig_index += 1;
            if sig_index == num_of_sigs as usize {
                break;
            }
        }
    }
    keys_and_signatures
}

// Check for correct input sizes and return the number of submitted signatures iff everything is
// valid.
fn sanity_check(bitmap: &BitVec<u32>, signatures: &ByteArray, pubkeys: &ByteArray) -> Result<u64> {
    let bitmap_len = bitmap.len();
    let signatures_len = signatures.len();
    let public_keys_len = pubkeys.len();

    // Ensure a BITMAP_SIZE bitmap.
    if bitmap_len != BITMAP_SIZE {
        bail!("Invalid bitmap length");
    }

    let mut bitmap_last_bit_set: usize = 0; // This is fine as we expect at least one set bit.
    let mut bitmap_count_ones: usize = 0;
    for (i, bit) in bitmap.iter().enumerate() {
        if bit {
            bitmap_count_ones += 1;
            bitmap_last_bit_set = i;
        }
    }
    if bitmap_count_ones == 0 {
        bail!("Bitmap is all zeros");
    }
    // Ensure we have as many signatures as the number of set bits in bitmap.
    if bitmap_count_ones * 64 != signatures_len {
        bail!("Mismatch between Bitmap Hamming weight and number of signatures");
    }
    // Ensure that we have at least as many keys as the index of the last set bit in bitmap.
    if public_keys_len < 32 * bitmap_last_bit_set {
        bail!("Bitmap points to a non-existent key");
    }
    // Ensure no more than BITMAP_SIZE keys.
    if public_keys_len > 32 * BITMAP_SIZE {
        bail!("Length of bytes of concatenated keys exceeds the maximum allowed");
    }
    // Ensure ByteArray for keys is a multiple of 32 bytes.
    if public_keys_len % 32 != 0 {
        bail!("Concatenated Ed25519 public keys should be a multiple of 32 bytes");
    }
    Ok(bitmap_count_ones as u64)
}
