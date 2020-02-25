// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    native_functions::dispatch::{native_gas, NativeResult},
    values::Value,
};
use libra_crypto::HashValue;
use libra_types::{
    byte_array::ByteArray,
    language_storage::TypeTag,
    vm_error::{StatusCode, VMStatus},
};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use vm::{
    errors::VMResult,
    gas_schedule::{CostTable, NativeCostIndex},
};

pub fn native_sha2_256(
    _ty_args: Vec<TypeTag>,
    mut arguments: VecDeque<Value>,
    cost_table: &CostTable,
) -> VMResult<NativeResult> {
    if arguments.len() != 1 {
        let msg = format!(
            "wrong number of arguments for sha2_256 expected 1 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = native_gas(cost_table, NativeCostIndex::SHA2_256, hash_arg.len());
    let hash_vec = Sha256::digest(hash_arg.as_bytes()).to_vec();
    let return_values = vec![Value::byte_array(ByteArray::new(hash_vec))];
    Ok(NativeResult::ok(cost, return_values))
}

pub fn native_sha3_256(
    _ty_args: Vec<TypeTag>,
    mut arguments: VecDeque<Value>,
    cost_table: &CostTable,
) -> VMResult<NativeResult> {
    if arguments.len() != 1 {
        let msg = format!(
            "wrong number of arguments for sha3_256 expected 1 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let hash_arg = pop_arg!(arguments, ByteArray);
    let cost = native_gas(cost_table, NativeCostIndex::SHA3_256, hash_arg.len());
    let hash_vec = HashValue::from_sha3_256(hash_arg.as_bytes()).to_vec();
    let return_values = vec![Value::byte_array(ByteArray::new(hash_vec))];
    Ok(NativeResult::ok(cost, return_values))
}
