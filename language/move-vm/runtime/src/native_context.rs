// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{interpreter::Interpreter, loader::Resolver};
use libra_types::{
    access_path::AccessPath, account_address::AccountAddress, contract_event::ContractEvent,
    language_storage::ModuleId,
};
use move_core_types::{gas_schedule::CostTable, identifier::IdentStr};
use move_vm_types::{
    interpreter_context::InterpreterContext,
    loaded_data::{runtime_types::Type, types::FatType},
    native_functions::context::NativeContext,
    values::Struct,
};
use std::fmt::Write;
use vm::errors::VMResult;

pub(crate) struct FunctionContext<'a, 'txn> {
    interpreter: &'a mut Interpreter<'txn>,
    interpreter_context: &'a mut dyn InterpreterContext,
    resolver: &'a Resolver<'a>,
}

impl<'a, 'txn> FunctionContext<'a, 'txn> {
    pub fn new(
        interpreter: &'a mut Interpreter<'txn>,
        context: &'a mut dyn InterpreterContext,
        resolver: &'a Resolver<'a>,
    ) -> FunctionContext<'a, 'txn> {
        FunctionContext {
            interpreter,
            interpreter_context: context,
            resolver,
        }
    }
}

impl<'a, 'txn> NativeContext for FunctionContext<'a, 'txn> {
    fn print_stack_trace<B: Write>(&self, buf: &mut B) -> VMResult<()> {
        self.interpreter
            .debug_print_stack_trace(buf, &self.resolver)
    }

    fn cost_table(&self) -> &CostTable {
        self.interpreter.gas_schedule()
    }

    fn save_under_address(
        &mut self,
        ty_args: &[Type],
        module_id: &ModuleId,
        struct_name: &IdentStr,
        resource_to_save: Struct,
        account_address: AccountAddress,
    ) -> VMResult<()> {
        let libra_type = self.resolver.get_libra_type_info(
            module_id,
            struct_name,
            ty_args,
            self.interpreter_context,
        )?;
        let ap = AccessPath::new(account_address, libra_type.resource_key().to_vec());
        self.interpreter_context
            .move_resource_to(&ap, libra_type.fat_type(), resource_to_save)
    }

    fn save_event(&mut self, event: ContractEvent) {
        self.interpreter_context.push_event(event)
    }

    fn convert_to_fat_types(&self, types: Vec<Type>) -> VMResult<Vec<FatType>> {
        types
            .iter()
            .map(|ty| self.resolver.type_to_fat_type(ty))
            .collect()
    }
}
