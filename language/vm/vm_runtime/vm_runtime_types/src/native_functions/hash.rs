// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::dispatch::NativeReturnStatus;
use crate::value::Value;
use crypto::HashValue;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use types::byte_array::ByteArray;

const SHA2_COST: u64 = 30;
const SHA3_COST: u64 = 30;

pub fn native_sha2_256(mut arguments: VecDeque<Value>) -> NativeReturnStatus {
    if arguments.len() != 1 {
        return NativeReturnStatus::InvalidArguments;
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = SHA2_COST * hash_arg.len() as u64;

    let hash_vec = Sha256::digest(hash_arg.as_bytes()).to_vec();
    let return_values = vec![Value::byte_array(ByteArray::new(hash_vec))];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}

pub fn native_sha3_256(mut arguments: VecDeque<Value>) -> NativeReturnStatus {
    if arguments.len() != 1 {
        return NativeReturnStatus::InvalidArguments;
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = SHA3_COST * hash_arg.len() as u64;

    let hash_vec = HashValue::from_sha3_256(hash_arg.as_bytes()).to_vec();
    let return_values = vec![Value::byte_array(ByteArray::new(hash_vec))];
    NativeReturnStatus::Success {
        cost,
        return_values,
    }
}
