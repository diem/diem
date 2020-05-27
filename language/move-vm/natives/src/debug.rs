// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_schedule::ZERO_GAS_UNITS;
#[allow(unused_imports)]
use move_vm_types::values::{values_impl::debug::print_reference, Reference};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{NativeContext, NativeResult},
    values::Value,
};
use std::collections::VecDeque;
use vm::errors::VMResult;

#[allow(unused_mut)]
#[allow(unused_variables)]
pub fn native_print(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    // No-op if the feature flag is not present.
    #[cfg(feature = "debug_module")]
    {
        let mut ty_args = context.convert_to_fat_types(ty_args)?;
        let ty = ty_args.pop().unwrap();
        let r = pop_arg!(args, Reference);

        let mut buf = String::new();
        print_reference(&mut buf, &ty, &r)?;
        println!("[debug] {}", buf);
    }

    Ok(NativeResult::ok(ZERO_GAS_UNITS, vec![]))
}

#[allow(unused_variables)]
pub fn native_print_stack_trace(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.is_empty());

    #[cfg(feature = "debug_module")]
    {
        let mut s = String::new();
        context.print_stack_trace(&mut s)?;
        println!("{}", s);
    }

    Ok(NativeResult::ok(ZERO_GAS_UNITS, vec![]))
}
