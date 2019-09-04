// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::dispatch::NativeReturnStatus;
use crate::value::Local;
use bitcoin_hashes::{hash160, sha256, Hash};
use crypto::HashValue;
use std::{borrow::Borrow, collections::VecDeque};
use tiny_keccak::Keccak;
use types::byte_array::ByteArray;

const HASH_LENGTH: usize = 32;
const KECCAK_COST: u64 = 30;
const RIPEMD_COST: u64 = 35;
const SHA2_COST: u64 = 30;
const SHA3_COST: u64 = 30;

pub fn native_keccak_256(mut arguments: VecDeque<Local>) -> NativeReturnStatus {
    if arguments.len() != 1 {
        return NativeReturnStatus::InvalidArguments;
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = KECCAK_COST * hash_arg.len() as u64;

    let mut hash = [0u8; HASH_LENGTH];
    let mut keccak = Keccak::new_keccak256();
    keccak.update(hash_arg.as_bytes());
    keccak.finalize(&mut hash);

    let return_values = vec![Local::bytearray(ByteArray::new(hash.to_vec()))];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

pub fn native_ripemd_160(mut arguments: VecDeque<Local>) -> NativeReturnStatus {
    if arguments.len() != 1 {
        return NativeReturnStatus::InvalidArguments;
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = RIPEMD_COST * hash_arg.len() as u64;

    let hash = hash160::Hash::hash(hash_arg.as_bytes());
    let hash_ref: &[u8] = hash.borrow();
    let return_values = vec![Local::bytearray(ByteArray::new(hash_ref.to_vec()))];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

pub fn native_sha2_256(mut arguments: VecDeque<Local>) -> NativeReturnStatus {
    if arguments.len() != 1 {
        return NativeReturnStatus::InvalidArguments;
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = SHA2_COST * hash_arg.len() as u64;

    let hash = sha256::Hash::hash(hash_arg.as_bytes());
    let hash_ref: &[u8] = hash.borrow();
    let return_values = vec![Local::bytearray(ByteArray::new(hash_ref.to_vec()))];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

pub fn native_sha3_256(mut arguments: VecDeque<Local>) -> NativeReturnStatus {
    if arguments.len() != 1 {
        return NativeReturnStatus::InvalidArguments;
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = SHA3_COST * hash_arg.len() as u64;

    let hash_value = HashValue::from_sha3_256(hash_arg.as_bytes());
    let return_values = vec![Local::bytearray(ByteArray::new(hash_value.to_vec()))];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}
