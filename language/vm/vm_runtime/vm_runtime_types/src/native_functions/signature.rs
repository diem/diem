// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::dispatch::NativeReturnStatus;
use crate::value::Local;
use bit_vec::BitVec;
use nextgen_crypto::{
    ed25519::{self, Ed25519PublicKey, Ed25519Signature},
    traits::*,
    HashValue,
};
use std::{collections::VecDeque, convert::TryFrom};
use types::byte_array::ByteArray;

// TODO: Talk to Crypto to determine these costs
const ED25519_COST: u64 = 35;
const BATCH_ED25519_COST: u64 = 30;

const BITMAP_SIZE: usize = 32;

/// Starting error code number
const DEFAULT_ERROR_CODE: u64 = 0x0ED2_5519;
/// Batch signature verification failed
const SIGNATURE_VERIFICATION_FAILURE: u64 = DEFAULT_ERROR_CODE + 1;
/// Public keys deserialization error
const PUBLIC_KEY_DESERIALIZATION_FAILURE: u64 = DEFAULT_ERROR_CODE + 2;
/// Signatures deserialization error
const SIGNATURE_DESERIALIZATION_FAILURE: u64 = DEFAULT_ERROR_CODE + 3;
/// Bitmap is all zeros
const ZERO_BITMAP_FAILURE: u64 = DEFAULT_ERROR_CODE + 4;
/// Invalid bitmap length
const INVALID_BITMAP_LENGTH_FAILURE: u64 = DEFAULT_ERROR_CODE + 5;
/// Mismatch between bitmap's Hamming weight and number or size of signatures
const SIGNATURE_SIZE_FAILURE: u64 = DEFAULT_ERROR_CODE + 6;
/// Bitmap points to a non-existent key
const BITMAP_PUBLIC_KEY_SIZE_FAILURE: u64 = DEFAULT_ERROR_CODE + 7;
/// Length of bytes of concatenated keys exceeds the maximum allowed
const OVERSIZED_PUBLIC_KEY_SIZE_FAILURE: u64 = DEFAULT_ERROR_CODE + 8;
/// Concatenated Ed25519 public keys should be a multiple of 32 bytes
const INVALID_PUBLIC_KEY_SIZE_FAILURE: u64 = DEFAULT_ERROR_CODE + 9;

pub fn native_ed25519_signature_verification(mut arguments: VecDeque<Local>) -> NativeReturnStatus {
    if arguments.len() != 3 {
        return NativeReturnStatus::InvalidArguments;
    }
    let msg = pop_arg!(arguments, ByteArray);
    let pubkey = pop_arg!(arguments, ByteArray);
    let signature = pop_arg!(arguments, ByteArray);

    let cost = ED25519_COST * msg.len() as u64;

    let sig = match ed25519::Ed25519Signature::try_from(signature.as_bytes()) {
        Ok(sig) => sig,
        Err(_) => {
            return NativeReturnStatus::Aborted {
                cost,
                error_code: DEFAULT_ERROR_CODE,
            }
        }
    };
    let pk = match ed25519::Ed25519PublicKey::try_from(pubkey.as_bytes()) {
        Ok(pk) => pk,
        Err(_) => {
            return NativeReturnStatus::Aborted {
                cost,
                error_code: DEFAULT_ERROR_CODE,
            }
        }
    };

    let bool_value = match sig.verify_arbitrary_msg(msg.as_bytes(), &pk) {
        Ok(()) => true,
        Err(_) => false,
    };
    let return_values = vec![Local::bool(bool_value)];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

/// Batch verify a collection of signatures using a bitmap for matching signatures to keys.
pub fn native_ed25519_threshold_signature_verification(
    mut arguments: VecDeque<Local>,
) -> NativeReturnStatus {
    if arguments.len() != 4 {
        return NativeReturnStatus::InvalidArguments;
    }
    let message = pop_arg!(arguments, ByteArray);
    let public_keys = pop_arg!(arguments, ByteArray);
    let signatures = pop_arg!(arguments, ByteArray);
    let bitmap = pop_arg!(arguments, ByteArray);

    let num_of_sigs = match ed25519_threshold_signature_verification(
        &bitmap,
        &signatures,
        &public_keys,
        &message,
        BATCH_ED25519_COST,
    ) {
        Ok(num_of_sigs) => num_of_sigs,
        Err(e) => return e,
    };

    let cost = ed25519_threshold_signature_verification_cost(num_of_sigs, message.len());
    let return_values = vec![Local::u64(num_of_sigs)];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

fn ed25519_threshold_signature_verification_cost(num_of_sigs: u64, message_len: usize) -> u64 {
    BATCH_ED25519_COST * num_of_sigs * message_len as u64
}

fn ed25519_threshold_signature_verification(
    bitmap: &ByteArray,
    signatures: &ByteArray,
    public_keys: &ByteArray,
    message: &ByteArray,
    abort_cost: u64,
) -> std::result::Result<u64, NativeReturnStatus> {
    let bitvec = BitVec::from_bytes(bitmap.as_bytes());

    let num_of_sigs = sanity_check(&bitvec, &signatures, &public_keys, abort_cost)?;
    let abort_cost = ed25519_threshold_signature_verification_cost(num_of_sigs, message.len());

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
                    let hash_value = match HashValue::from_slice(message.as_bytes()) {
                        Err(_) => {
                            return Err(NativeReturnStatus::Aborted {
                                cost: abort_cost,
                                error_code: DEFAULT_ERROR_CODE,
                            })
                        }
                        Ok(hash_value) => hash_value,
                    };
                    match Ed25519Signature::batch_verify_signatures(
                        &hash_value,
                        keys_and_signatures,
                    ) {
                        Ok(()) => Ok(num_of_sigs),
                        Err(_) =>
                        // Batch verification failed
                        {
                            Err(NativeReturnStatus::Aborted {
                                cost: abort_cost,
                                error_code: SIGNATURE_VERIFICATION_FAILURE,
                            })
                        }
                    }
                }
                Err(_) =>
                // Key deserialization error
                {
                    Err(NativeReturnStatus::Aborted {
                        cost: abort_cost,
                        error_code: PUBLIC_KEY_DESERIALIZATION_FAILURE,
                    })
                }
            }
        }
        Err(_) =>
        // Signature deserialization error
        {
            Err(NativeReturnStatus::Aborted {
                cost: abort_cost,
                error_code: SIGNATURE_DESERIALIZATION_FAILURE,
            })
        }
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
fn sanity_check(
    bitmap: &BitVec<u32>,
    signatures: &ByteArray,
    pubkeys: &ByteArray,
    abort_cost: u64,
) -> std::result::Result<u64, NativeReturnStatus> {
    let bitmap_len = bitmap.len();
    let signatures_len = signatures.len();
    let public_keys_len = pubkeys.len();

    // Ensure a BITMAP_SIZE bitmap.
    if bitmap_len != BITMAP_SIZE {
        // Invalid bitmap length
        return Err(NativeReturnStatus::Aborted {
            cost: abort_cost,
            error_code: INVALID_BITMAP_LENGTH_FAILURE,
        });
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
        // Bitmap is all zeros
        return Err(NativeReturnStatus::Aborted {
            cost: abort_cost,
            error_code: ZERO_BITMAP_FAILURE,
        });
    }
    // Ensure we have as many signatures as the number of set bits in bitmap.
    if bitmap_count_ones * 64 != signatures_len {
        // Mismatch between Bitmap Hamming weight and number of signatures
        return Err(NativeReturnStatus::Aborted {
            cost: abort_cost,
            error_code: SIGNATURE_SIZE_FAILURE,
        });
    }
    // Ensure that we have at least as many keys as the index of the last set bit in bitmap.
    if public_keys_len < 32 * (bitmap_last_bit_set + 1) {
        // Bitmap points to a non-existent key
        return Err(NativeReturnStatus::Aborted {
            cost: abort_cost,
            error_code: BITMAP_PUBLIC_KEY_SIZE_FAILURE,
        });
    }
    // Ensure no more than BITMAP_SIZE keys.
    if public_keys_len > 32 * BITMAP_SIZE {
        // Length of bytes of concatenated keys exceeds the maximum allowed
        return Err(NativeReturnStatus::Aborted {
            cost: abort_cost,
            error_code: OVERSIZED_PUBLIC_KEY_SIZE_FAILURE,
        });
    }
    // Ensure ByteArray for keys is a multiple of 32 bytes.
    if public_keys_len % 32 != 0 {
        // Concatenated Ed25519 public keys should be a multiple of 32 bytes
        return Err(NativeReturnStatus::Aborted {
            cost: abort_cost,
            error_code: INVALID_PUBLIC_KEY_SIZE_FAILURE,
        });
    }
    Ok(bitmap_count_ones as u64)
}
