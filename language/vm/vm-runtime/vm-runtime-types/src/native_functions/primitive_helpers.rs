// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    native_functions::dispatch::{native_gas, NativeResult},
    value::Value,
};
use libra_types::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    vm_error::{StatusCode, VMStatus},
};
use std::collections::VecDeque;
use vm::{
    errors::VMResult,
    gas_schedule::{CostTable, NativeCostIndex},
};

pub fn native_bytearray_concat(
    mut arguments: VecDeque<Value>,
    cost_table: &CostTable,
) -> VMResult<NativeResult> {
    if arguments.len() != 2 {
        let msg = format!(
            "wrong number of arguments for bytearray_concat expected 2 found {}",
            arguments.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    let arg2 = pop_arg!(arguments, ByteArray);
    let arg1 = pop_arg!(arguments, ByteArray);
    let mut return_val = arg1.as_bytes().to_vec();
    return_val.extend_from_slice(arg2.as_bytes());

    let cost = native_gas(
        cost_table,
        NativeCostIndex::BYTEARRAY_CONCAT,
        return_val.len(),
    );
    let return_values = vec![Value::byte_array(ByteArray::new(return_val))];
    Ok(NativeResult::ok(cost, return_values))
}

pub fn native_address_to_bytes(
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
    let return_values = vec![Value::byte_array(ByteArray::new(return_val))];
    Ok(NativeResult::ok(cost, return_values))
}

pub fn native_u64_to_bytes(
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
    let return_values = vec![Value::byte_array(ByteArray::new(return_val))];
    Ok(NativeResult::ok(cost, return_values))
}
