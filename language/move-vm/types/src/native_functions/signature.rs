// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::runtime_types::Type,
    native_functions::{
        context::NativeContext,
        dispatch::{native_gas, NativeResult},
    },
    values::Value,
};
use bit_vec::BitVec;
use libra_crypto::{
    ed25519::{self, Ed25519PublicKey, Ed25519Signature},
    traits::*,
    HashValue,
};
use libra_types::vm_error::{StatusCode, VMStatus};
use move_core_types::gas_schedule::{CostTable, NativeCostIndex};
use std::{collections::VecDeque, convert::TryFrom};
use vm::errors::VMResult;

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

pub fn native_ed25519_signature_verification(
    context: &impl NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    if arguments.len() != 3 {
        let msg = format!(
            "wrong number of arguments for ed25519_signature_verification expected 3 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let msg = pop_arg!(arguments, Vec<u8>);
    let pubkey = pop_arg!(arguments, Vec<u8>);
    let signature = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::ED25519_VERIFY,
        msg.len(),
    );

    let sig = match ed25519::Ed25519Signature::try_from(signature.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return Ok(NativeResult::err(
                cost,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(DEFAULT_ERROR_CODE),
            ));
        }
    };
    let pk = match ed25519::Ed25519PublicKey::try_from(pubkey.as_slice()) {
        Ok(pk) => pk,
        Err(_) => {
            return Ok(NativeResult::err(
                cost,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(DEFAULT_ERROR_CODE),
            ));
        }
    };

    let bool_value = sig.verify_arbitrary_msg(msg.as_slice(), &pk).is_ok();
    let return_values = vec![Value::bool(bool_value)];
    Ok(NativeResult::ok(cost, return_values))
}
// TODO: activate the batch feature in the import of libra-crypto in the
// present crate, once https://github.com/libra/libra/issues/3567 is fixed
/// Batch verify a collection of signatures using a bitmap for matching signatures to keys.
pub fn native_ed25519_threshold_signature_verification(
    context: &impl NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    if arguments.len() != 4 {
        let msg = format!(
            "wrong number of arguments for ed25519_threshold_signature_verification expected 4 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let message = pop_arg!(arguments, Vec<u8>);
    let public_keys = pop_arg!(arguments, Vec<u8>);
    let signatures = pop_arg!(arguments, Vec<u8>);
    let bitmap = pop_arg!(arguments, Vec<u8>);

    Ok(ed25519_threshold_signature_verification(
        &bitmap,
        &signatures,
        &public_keys,
        &message,
        context.cost_table(),
    ))
}

fn ed25519_threshold_signature_verification(
    bitmap: &[u8],
    signatures: &[u8],
    public_keys: &[u8],
    message: &[u8],
    cost_table: &CostTable,
) -> NativeResult {
    let bitvec = BitVec::from_bytes(bitmap);

    let num_of_sigs = match sanity_check(&bitvec, &signatures, &public_keys, cost_table) {
        Ok(sig_count) => sig_count,
        Err(result) => return result,
    };
    let cost = native_gas(
        cost_table,
        NativeCostIndex::ED25519_THRESHOLD_VERIFY,
        num_of_sigs as usize * message.len(),
    );

    let sig_chunks: ::std::result::Result<Vec<_>, _> = signatures
        .chunks(64)
        .map(Ed25519Signature::try_from)
        .collect();

    match sig_chunks {
        Ok(signatures) => {
            let key_chunks: ::std::result::Result<Vec<_>, _> = public_keys
                .chunks(32)
                .map(Ed25519PublicKey::try_from)
                .collect();

            match key_chunks {
                Ok(keys) => {
                    let keys_and_signatures =
                        matching_keys_and_signatures(num_of_sigs, bitvec, signatures, keys);
                    let hash_value = match HashValue::from_slice(message) {
                        Err(_) => {
                            return NativeResult::err(
                                cost,
                                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                                    .with_sub_status(DEFAULT_ERROR_CODE),
                            )
                        }
                        Ok(hash_value) => hash_value,
                    };
                    match Ed25519Signature::batch_verify_signatures(
                        &hash_value,
                        keys_and_signatures,
                    ) {
                        Ok(()) => NativeResult::ok(cost, vec![Value::u64(num_of_sigs)]),
                        Err(_) =>
                        // Batch verification failed
                        {
                            NativeResult::err(
                                cost,
                                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                                    .with_sub_status(SIGNATURE_VERIFICATION_FAILURE),
                            )
                        }
                    }
                }
                Err(_) =>
                // Key deserialization error
                {
                    NativeResult::err(
                        cost,
                        VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(PUBLIC_KEY_DESERIALIZATION_FAILURE),
                    )
                }
            }
        }
        Err(_) =>
        // Signature deserialization error
        {
            NativeResult::err(
                cost,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(SIGNATURE_DESERIALIZATION_FAILURE),
            )
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
    signatures: &[u8],
    pubkeys: &[u8],
    cost_table: &CostTable,
) -> std::result::Result<u64, NativeResult> {
    let bitmap_len = bitmap.len();
    let signatures_len = signatures.len();
    let public_keys_len = pubkeys.len();

    let cost = native_gas(
        cost_table,
        NativeCostIndex::ED25519_THRESHOLD_VERIFY,
        bitmap_len + signatures_len + public_keys_len,
    );

    // Ensure a BITMAP_SIZE bitmap.
    if bitmap_len != BITMAP_SIZE {
        // Invalid bitmap length
        return Err(NativeResult::err(
            cost,
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(INVALID_BITMAP_LENGTH_FAILURE),
        ));
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
        return Err(NativeResult::err(
            cost,
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR).with_sub_status(ZERO_BITMAP_FAILURE),
        ));
    }
    // Ensure we have as many signatures as the number of set bits in bitmap.
    if bitmap_count_ones * 64 != signatures_len {
        // Mismatch between Bitmap Hamming weight and number of signatures
        return Err(NativeResult::err(
            cost,
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(SIGNATURE_SIZE_FAILURE),
        ));
    }
    // Ensure that we have at least as many keys as the index of the last set bit in bitmap.
    if public_keys_len < 32 * (bitmap_last_bit_set + 1) {
        // Bitmap points to a non-existent key
        return Err(NativeResult::err(
            cost,
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(BITMAP_PUBLIC_KEY_SIZE_FAILURE),
        ));
    }
    // Ensure no more than BITMAP_SIZE keys.
    if public_keys_len > 32 * BITMAP_SIZE {
        // Length of bytes of concatenated keys exceeds the maximum allowed
        return Err(NativeResult::err(
            cost,
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(OVERSIZED_PUBLIC_KEY_SIZE_FAILURE),
        ));
    }
    // Ensure ByteArray for keys is a multiple of 32 bytes.
    if public_keys_len % 32 != 0 {
        // Concatenated Ed25519 public keys should be a multiple of 32 bytes
        return Err(NativeResult::err(
            cost,
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(INVALID_PUBLIC_KEY_SIZE_FAILURE),
        ));
    }
    Ok(bitmap_count_ones as u64)
}
