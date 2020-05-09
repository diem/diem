// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::account_address::AccountAddress;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeResult},
    values::Value,
};
use std::collections::VecDeque;
use vm::errors::VMResult;

pub fn native_create_signer(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let address = pop_arg!(arguments, AccountAddress);
    let cost = native_gas(context.cost_table(), NativeCostIndex::CREATE_SIGNER, 0);
    Ok(NativeResult::ok(cost, vec![Value::signer(address)]))
}

pub fn native_destroy_signer(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let cost = native_gas(context.cost_table(), NativeCostIndex::CREATE_SIGNER, 0);
    Ok(NativeResult::ok(cost, vec![]))
}
