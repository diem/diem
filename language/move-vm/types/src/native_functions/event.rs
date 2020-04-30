// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::runtime_types::Type,
    native_functions::{context::NativeContext, dispatch::NativeResult},
    values::Value,
};
use libra_types::{
    contract_event::ContractEvent,
    event::EventKey,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::gas_schedule::ZERO_GAS_UNITS;
use std::{collections::VecDeque, convert::TryFrom};
use vm::errors::VMResult;

pub fn native_emit_event(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    if ty_args.len() != 1 {
        return Err(
            VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(format!(
                "write_to_event_storage expects 1 type argument got {}.",
                ty_args.len()
            )),
        );
    }

    let mut ty_args = context.convert_to_fat_types(ty_args)?;
    let ty = ty_args.pop().unwrap();

    let msg = arguments
        .pop_back()
        .unwrap()
        .simple_serialize(&ty)
        .ok_or_else(|| VMStatus::new(StatusCode::DATA_FORMAT_ERROR))?;

    let count = pop_arg!(arguments, u64);
    let key = pop_arg!(arguments, Vec<u8>);
    let guid = EventKey::try_from(key.as_slice())
        .map_err(|_| VMStatus::new(StatusCode::EVENT_KEY_MISMATCH))?;
    context.save_event(ContractEvent::new(guid, count, ty.type_tag()?, msg))?;

    Ok(NativeResult::ok(ZERO_GAS_UNITS, vec![]))
}
