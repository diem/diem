// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    interpreter_context::InterpreterContext, loaded_data::loaded_module::LoadedModule,
    runtime::VMRuntime,
};
use bytecode_verifier::VerifiedModule;
use libra_types::language_storage::ModuleId;
use move_core_types::identifier::{IdentStr, Identifier};
use move_vm_cache::Arena;
use move_vm_definition::MoveVMImpl;
use move_vm_types::{
    chain_state::ChainState,
    loaded_data::types::{StructType, Type},
    values::Value,
};
use vm::{errors::VMResult, gas_schedule::CostTable, transaction_metadata::TransactionMetadata};

rental! {
    mod move_vm_definition {
        use super::*;

        #[rental]
        pub(super) struct MoveVMImpl {
            alloc: Box<Arena<LoadedModule>>,
            runtime: VMRuntime<'alloc>,
        }
    }
}

pub struct MoveVM(MoveVMImpl);

impl MoveVM {
    pub fn new() -> Self {
        MoveVM(MoveVMImpl::new(Box::new(Arena::new()), |arena| {
            VMRuntime::new(&*arena)
        }))
    }

    pub fn execute_function<S: ChainState>(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        gas_schedule: &CostTable,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
        args: Vec<Value>,
    ) -> VMResult<()> {
        self.0.rent(|runtime| {
            runtime.execute_function(
                chain_state,
                txn_data,
                gas_schedule,
                module,
                function_name,
                args,
            )
        })
    }

    #[allow(unused)]
    pub fn execute_script<S: ChainState>(
        &self,
        script: Vec<u8>,
        gas_schedule: &CostTable,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
        args: Vec<Value>,
    ) -> VMResult<()> {
        self.0.rent(|runtime| {
            runtime.execute_script(chain_state, txn_data, gas_schedule, script, args)
        })
    }

    pub fn publish_module<S: ChainState>(
        &self,
        module: Vec<u8>,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        self.0
            .rent(|runtime| runtime.publish_module(module, chain_state, txn_data))
    }

    pub fn cache_module(&self, module: VerifiedModule) {
        self.0.rent(|runtime| runtime.cache_module(module))
    }

    pub fn resolve_struct_def_by_name<S: ChainState>(
        &self,
        module_id: &ModuleId,
        name: &Identifier,
        chain_state: &mut S,
        ty_args: &[Type],
    ) -> VMResult<StructType> {
        self.0.rent(|runtime| {
            runtime.resolve_struct_def_by_name(module_id, name, ty_args, chain_state)
        })
    }

    /// This is an internal method that is exposed only for tests and cost synthesis.
    /// TODO: Figure out a better way to do this.
    pub fn get_loaded_module(
        &self,
        id: &ModuleId,
        data_view: &dyn InterpreterContext,
    ) -> VMResult<&LoadedModule> {
        self.0
            .try_ref_rent(|runtime| runtime.get_loaded_module(id, data_view))
    }

    #[cfg(any(test, feature = "instruction_synthesis"))]
    pub(crate) fn with_runtime<T>(&self, f: impl FnOnce(&VMRuntime) -> T) -> T {
        self.0.rent(|runtime| f(runtime))
    }
}

impl Default for MoveVM {
    fn default() -> Self {
        Self::new()
    }
}
