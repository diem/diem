// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::account_address::AccountAddress;
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
pub struct NativeAccountCreateSigner;
impl NativeFunction for NativeAccountCreateSigner {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.is_empty());
        debug_assert!(args.len() == 1);

        let address = pop_arg!(args, AccountAddress);
        let cost = native_gas(context.cost_table(), NativeCostIndex::CREATE_SIGNER, 0);
        Ok(NativeResult::ok(cost, smallvec![Value::signer(address)]))
    }
}

/// NOTE: this function will be deprecated after the Diem v3 release, but must
/// remain for replaying old transactions
#[derive(Copy, Clone)]
pub struct NativeAccountDestroySigner;
impl NativeFunction for NativeAccountDestroySigner {
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
        debug_assert!(args.len() == 1);

        let cost = native_gas(context.cost_table(), NativeCostIndex::DESTROY_SIGNER, 0);
        Ok(NativeResult::ok(cost, smallvec![]))
    }
}
