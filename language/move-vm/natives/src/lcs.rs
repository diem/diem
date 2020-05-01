// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::vm_error::{sub_status::NFE_LCS_SERIALIZATION_FAILURE, StatusCode, VMStatus};
use move_core_types::gas_schedule::NativeCostIndex;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeResult},
    values::{values_impl::Reference, Value},
};
use std::collections::VecDeque;
use vm::errors::VMResult;

/// Rust implementation of Move's `native public fun to_bytes<T>(&T): vector<u8>`
pub fn native_to_bytes(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let ref_to_val = pop_arg!(args, Reference);

    let mut ty_args = context.convert_to_fat_types(ty_args)?;
    let arg_type = ty_args.pop().unwrap();
    // delegate to the LCS serialization for `Value`
    let serialized_value = ref_to_val
        .read_ref()?
        .simple_serialize(&arg_type)
        .ok_or_else(|| {
            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                .with_sub_status(NFE_LCS_SERIALIZATION_FAILURE)
        })?;
    // cost is proportional to the size of the serialized value
    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::LCS_TO_BYTES,
        serialized_value.len(),
    );

    Ok(NativeResult::ok(
        cost,
        vec![Value::vector_u8(serialized_value)],
    ))
}
