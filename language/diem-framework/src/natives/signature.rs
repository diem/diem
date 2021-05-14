// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{ed25519, traits::Signature};
use move_binary_format::errors::PartialVMResult;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeFunction, NativeResult},
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::convert::TryFrom;

#[derive(Copy, Clone)]
pub struct NativeSignatureEd25519ValidatePubkey;
impl NativeFunction for NativeSignatureEd25519ValidatePubkey {
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

        let key_bytes = pop_arg!(args, Vec<u8>);

        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::ED25519_VALIDATE_KEY,
            key_bytes.len(),
        );

        // This deserialization performs point-on-curve and small subgroup checks
        let valid = ed25519::Ed25519PublicKey::try_from(&key_bytes[..]).is_ok();
        Ok(NativeResult::ok(cost, smallvec![Value::bool(valid)]))
    }
}

#[derive(Copy, Clone)]
pub struct NativeSignatureEd25519Verify;
impl NativeFunction for NativeSignatureEd25519Verify {
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
        debug_assert!(args.len() == 3);

        let msg = pop_arg!(args, Vec<u8>);
        let pubkey = pop_arg!(args, Vec<u8>);
        let signature = pop_arg!(args, Vec<u8>);

        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::ED25519_VERIFY,
            msg.len(),
        );

        let sig = match ed25519::Ed25519Signature::try_from(signature.as_slice()) {
            Ok(sig) => sig,
            Err(_) => {
                return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
            }
        };
        let pk = match ed25519::Ed25519PublicKey::try_from(pubkey.as_slice()) {
            Ok(pk) => pk,
            Err(_) => {
                return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
            }
        };

        let verify_result = sig.verify_arbitrary_msg(msg.as_slice(), &pk).is_ok();
        Ok(NativeResult::ok(
            cost,
            smallvec![Value::bool(verify_result)],
        ))
    }
}
