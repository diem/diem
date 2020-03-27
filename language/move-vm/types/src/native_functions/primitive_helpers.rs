// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::types::Type,
    native_functions::dispatch::{native_gas, NativeResult},
    values::Value,
};
use libra_types::{
    account_address::AccountAddress,
    vm_error::{StatusCode, VMStatus},
};
use std::collections::VecDeque;
use vm::{
    errors::VMResult,
    gas_schedule::{CostTable, NativeCostIndex},
};

pub fn native_address_to_bytes(
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
    cost_table: &CostTable,
) -> VMResult<NativeResult> {
    if arguments.len() != 1 {
        let msg = format!(
            "wrong number of arguments for address_to_bytes expected 1 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let arg = pop_arg!(arguments, AccountAddress);
    let return_val = arg.to_vec();

    let cost = native_gas(
        cost_table,
        NativeCostIndex::ADDRESS_TO_BYTES,
        return_val.len(),
    );
    let return_values = vec![Value::vector_u8(return_val)];
    Ok(NativeResult::ok(cost, return_values))
}

pub fn native_u64_to_bytes(
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
    cost_table: &CostTable,
) -> VMResult<NativeResult> {
    if arguments.len() != 1 {
        let msg = format!(
            "wrong number of arguments for u64_to_bytes expected 1 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let arg = pop_arg!(arguments, u64);
    let return_val: Vec<u8> = arg.to_le_bytes().to_vec();

    let cost = native_gas(cost_table, NativeCostIndex::U64_TO_BYTES, return_val.len());
    let return_values = vec![Value::vector_u8(return_val)];
    Ok(NativeResult::ok(cost, return_values))
}
