// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::types::Type,
    native_functions::dispatch::{native_gas, NativeResult},
    values::{values_impl::Reference, Value},
};
use libra_types::vm_error::{sub_status::NFE_LCS_SERIALIZATION_FAILURE, StatusCode, VMStatus};
use std::collections::VecDeque;
use vm::{
    errors::VMResult,
    gas_schedule::{CostTable, NativeCostIndex},
};

/// Rust implementation of Move's `native public fun to_bytes<T>(&T): vector<u8>`
pub fn native_to_bytes(
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
    cost_table: &CostTable,
) -> VMResult<NativeResult> {
    if ty_args.len() != 1 {
        let msg = format!(
            "Wrong number of type arguments for serialize. Expected 1, but found {}",
            ty_args.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }
    if args.len() != 1 {
        let msg = format!(
            "Wrong number of arguments for serialize. Expected 1, but found {}",
            args.len()
        );
        return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
    }

    let arg_type = ty_args.pop().unwrap();
    // delegate to the LCS serialization for `Value`
    let ref_to_val = pop_arg!(args, Reference);
    let serialized_value = ref_to_val
        .read_ref()?
        .simple_serialize(&arg_type)
        .ok_or_else(|| {
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(NFE_LCS_SERIALIZATION_FAILURE)
        })?;
    // cost is proportional to the size of the serialized value
    let cost = native_gas(
        cost_table,
        NativeCostIndex::LCS_TO_BYTES,
        serialized_value.len(),
    );

    Ok(NativeResult::ok(
        cost,
        vec![Value::vector_u8(serialized_value)],
    ))
}
