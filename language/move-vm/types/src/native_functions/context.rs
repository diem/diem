// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{runtime_types::Type, types::FatType},
    values::Struct,
};
use libra_types::{
    account_address::AccountAddress, contract_event::ContractEvent, language_storage::ModuleId,
};
use move_core_types::{gas_schedule::CostTable, identifier::IdentStr};
use std::fmt::Write;
use vm::errors::VMResult;

/// Native function context.
pub trait NativeContext {
    /// Prints stack trace.
    fn print_stack_trace<B: Write>(&self, buf: &mut B) -> VMResult<()>;
    /// Gets cost table ref.
    fn cost_table(&self) -> &CostTable;
    // Save a resource under the address specified by `account_address`
    fn save_under_address(
        &mut self,
        ty_args: &[Type],
        module_id: &ModuleId,
        struct_name: &IdentStr,
        resource_to_save: Struct,
        account_address: AccountAddress,
    ) -> VMResult<()>;
    /// Saves contract event.
    fn save_event(&mut self, event: ContractEvent);
    /// Converts types to fet types.
    fn convert_to_fat_types(&self, types: Vec<Type>) -> VMResult<Vec<FatType>>;
}
