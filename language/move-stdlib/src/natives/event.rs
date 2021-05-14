// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_schedule::GasAlgebra;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeFunction, NativeResult},
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
pub struct NativeEventWriteToEventStore;
impl NativeFunction for NativeEventWriteToEventStore {
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
        debug_assert!(args.len() == 3);

        let ty = ty_args.pop().unwrap();
        let msg = args.pop_back().unwrap();
        let seq_num = pop_arg!(args, u64);
        let guid = pop_arg!(args, Vec<u8>);

        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::EMIT_EVENT,
            msg.size().get() as usize,
        );

        if !context.save_event(guid, seq_num, ty, msg)? {
            return Ok(NativeResult::err(cost, 0));
        }

        Ok(NativeResult::ok(cost, smallvec![]))
    }
}
