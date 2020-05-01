// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    account_address::AccountAddress,
    account_config,
    account_config::{AccountResource, BalanceResource, CORE_CODE_ADDRESS},
    move_resource::MoveResource,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::gas_schedule::NativeCostIndex;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeResult},
    values::{Struct, Value},
};
use std::collections::VecDeque;
use vm::errors::VMResult;

pub fn native_save_account(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> VMResult<NativeResult> {
    debug_assert!(ty_args.len() == 2);
    debug_assert!(arguments.len() == 5);

    let address = pop_arg!(arguments, AccountAddress);
    let event_generator = pop_arg!(arguments, Struct);
    let account = pop_arg!(arguments, Struct);
    let balance = pop_arg!(arguments, Struct);
    let account_type = pop_arg!(arguments, Struct);

    if address == CORE_CODE_ADDRESS {
        return Err(VMStatus::new(StatusCode::CREATE_NULL_ACCOUNT));
    }

    let cost = native_gas(context.cost_table(), NativeCostIndex::SAVE_ACCOUNT, 0);

    context.save_under_address(
        &[],
        &account_config::EVENT_MODULE,
        account_config::event_handle_generator_struct_name(),
        event_generator,
        address,
    )?;
    context.save_under_address(
        &[],
        &account_config::ACCOUNT_MODULE,
        &AccountResource::struct_identifier(),
        account,
        address,
    )?;
    context.save_under_address(
        &[ty_args[0].clone()],
        &account_config::ACCOUNT_MODULE,
        &BalanceResource::struct_identifier(),
        balance,
        address,
    )?;
    context.save_under_address(
        &[ty_args[1].clone()],
        &account_config::ACCOUNT_TYPE_MODULE,
        account_config::account_type_struct_name(),
        account_type,
        address,
    )?;
    Ok(NativeResult::ok(cost, vec![]))
}
