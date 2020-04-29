// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::runtime_types::Type,
    native_functions::{
        context::NativeContext,
        dispatch::{native_gas, NativeResult},
    },
    values::{Struct, Value},
};
use libra_types::{
    account_address::AccountAddress,
    account_config,
    account_config::{AccountResource, BalanceResource, CORE_CODE_ADDRESS},
    move_resource::MoveResource,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;
use vm::errors::VMResult;

pub fn native_save_account(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    let cost = native_gas(context.cost_table(), NativeCostIndex::SAVE_ACCOUNT, 0);

    let address = pop_arg!(arguments, AccountAddress);

    if address == CORE_CODE_ADDRESS {
        return Err(VMStatus::new(StatusCode::CREATE_NULL_ACCOUNT));
    }

    context.save_under_address(
        &[],
        &account_config::EVENT_MODULE,
        account_config::event_handle_generator_struct_name(),
        pop_arg!(arguments, Struct),
        address,
    )?;
    context.save_under_address(
        &[],
        &account_config::ACCOUNT_MODULE,
        &AccountResource::struct_identifier(),
        pop_arg!(arguments, Struct),
        address,
    )?;
    context.save_under_address(
        &[ty_args[0].clone()],
        &account_config::ACCOUNT_MODULE,
        &BalanceResource::struct_identifier(),
        pop_arg!(arguments, Struct),
        address,
    )?;
    context.save_under_address(
        &[ty_args[1].clone()],
        &account_config::ACCOUNT_TYPE_MODULE,
        account_config::account_type_struct_name(),
        pop_arg!(arguments, Struct),
        address,
    )?;
    Ok(NativeResult::ok(cost, vec![]))
}
