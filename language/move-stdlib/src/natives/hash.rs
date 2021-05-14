// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::HashValue;
use move_binary_format::errors::PartialVMResult;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeFunction, NativeResult},
    pop_arg,
    values::Value,
};
use sha2::{Digest, Sha256};
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
pub struct NativeHashSha2_256;
impl NativeFunction for NativeHashSha2_256 {
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

        let hash_arg = pop_arg!(args, Vec<u8>);

        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::SHA2_256,
            hash_arg.len(),
        );

        let hash_vec = Sha256::digest(hash_arg.as_slice()).to_vec();
        Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(hash_vec)],
        ))
    }
}

#[derive(Copy, Clone)]
pub struct NativeHashSha3_256;
impl NativeFunction for NativeHashSha3_256 {
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

        let hash_arg = pop_arg!(args, Vec<u8>);

        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::SHA3_256,
            hash_arg.len(),
        );

        let hash_vec = HashValue::sha3_256_of(hash_arg.as_slice()).to_vec();
        Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(hash_vec)],
        ))
    }
}
