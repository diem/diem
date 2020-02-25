// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chain_state::ChainState, loaded_data::loaded_module::LoadedModule, runtime::VMRuntime,
};
use bytecode_verifier::VerifiedModule;
use libra_types::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag},
};
use move_vm_definition::MoveVMImpl;
use move_vm_types::{loaded_data::struct_def::StructDef, values::Value};
use vm::{errors::VMResult, gas_schedule::CostTable, transaction_metadata::TransactionMetadata};
use vm_cache_map::Arena;

rental! {
    mod move_vm_definition {
        use super::*;

        #[rental]
        pub struct MoveVMImpl {
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

    pub fn resolve_struct_tag_by_name<S: ChainState>(
        &self,
        module_id: &ModuleId,
        name: &Identifier,
        chain_state: &mut S,
    ) -> VMResult<StructTag> {
        self.0
            .rent(|runtime| runtime.resolve_struct_tag_by_name(module_id, name, chain_state))
    }

    pub fn resolve_struct_def_by_name<S: ChainState>(
        &self,
        module_id: &ModuleId,
        name: &Identifier,
        chain_state: &mut S,
    ) -> VMResult<StructDef> {
        self.0
            .rent(|runtime| runtime.resolve_struct_def_by_name(module_id, name, chain_state))
    }
}

impl Default for MoveVM {
    fn default() -> Self {
        Self::new()
    }
}
