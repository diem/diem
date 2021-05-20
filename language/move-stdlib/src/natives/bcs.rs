// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::vm_status::sub_status::NFE_BCS_SERIALIZATION_FAILURE;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeFunction, NativeResult},
    pop_arg,
    values::{Reference, Value},
};
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
pub struct NativeBCSToBytes;
impl NativeFunction for NativeBCSToBytes {
    fn run<C>(
        &self,
        context: &mut C,
        mut ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 1);

        let ref_to_val = pop_arg!(args, Reference);

        let arg_type = ty_args.pop().unwrap();
        // delegate to the BCS serialization for `Value`
        let serialized_value_opt = match context.type_to_type_layout(&arg_type)? {
            None => None,
            Some(layout) => ref_to_val.read_ref()?.simple_serialize(&layout),
        };
        let serialized_value = match serialized_value_opt {
            None => {
                let cost = native_gas(context.cost_table(), NativeCostIndex::BCS_TO_BYTES, 1);
                return Ok(NativeResult::err(cost, NFE_BCS_SERIALIZATION_FAILURE));
            }
            Some(serialized_value) => serialized_value,
        };

        // cost is proportional to the size of the serialized value
        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::BCS_TO_BYTES,
            serialized_value.len(),
        );

        Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(serialized_value)],
        ))
    }
}
