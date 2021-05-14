// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_schedule::ONE_GAS_UNIT;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{NativeContext, NativeFunction, NativeResult},
    values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
#[allow(unused_variables)]
pub struct NativeDebugPrint;
impl NativeFunction for NativeDebugPrint {
    fn run<C>(
        &self,
        _context: &mut C,
        ty_args: Vec<Type>,
        args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 1);

        // No-op if the feature flag is not present.
        #[cfg(feature = "debug_module")]
        {
            let ty = ty_args.pop().unwrap();
            let r = pop_arg!(args, Reference);

            let mut buf = String::new();
            print_reference(&mut buf, &r)?;
            println!("[debug] {}", buf);
        }

        Ok(NativeResult::ok(ONE_GAS_UNIT, smallvec![]))
    }
}

#[derive(Copy, Clone)]
pub struct NativeDebugPrintStackTrace;
impl NativeFunction for NativeDebugPrintStackTrace {
    #[allow(unused_variables)]
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.is_empty());
        debug_assert!(args.is_empty());

        #[cfg(feature = "debug_module")]
        {
            let mut s = String::new();
            context.print_stack_trace(&mut s)?;
            println!("{}", s);
        }

        Ok(NativeResult::ok(ONE_GAS_UNIT, smallvec![]))
    }
}
