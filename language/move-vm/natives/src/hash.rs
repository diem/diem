// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::HashValue;
use move_core_types::gas_schedule::NativeCostIndex;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeResult},
    values::Value,
};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use vm::errors::VMResult;

pub fn native_sha2_256(
    context: &impl NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let hash_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::SHA2_256,
        hash_arg.len(),
    );

    let hash_vec = Sha256::digest(hash_arg.as_slice()).to_vec();
    let return_values = vec![Value::vector_u8(hash_vec)];
    Ok(NativeResult::ok(cost, return_values))
}

pub fn native_sha3_256(
    context: &impl NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let hash_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::SHA3_256,
        hash_arg.len(),
    );

    let hash_vec = HashValue::from_sha3_256(hash_arg.as_slice()).to_vec();
    let return_values = vec![Value::vector_u8(hash_vec)];
    Ok(NativeResult::ok(cost, return_values))
}
