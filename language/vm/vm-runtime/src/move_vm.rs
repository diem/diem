// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chain_state::ChainState, data_cache::RemoteCache, loaded_data::loaded_module::LoadedModule,
    runtime::VMRuntime,
};
use libra_state_view::StateView;
use libra_types::{identifier::IdentStr, language_storage::ModuleId};
use move_vm_definition::MoveVMImpl;
use vm::{errors::VMResult, gas_schedule::CostTable, transaction_metadata::TransactionMetadata};
use vm_cache_map::Arena;
use vm_runtime_types::value::Value;

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

    // API for temporal backward compatibility.
    // TODO: Get rid of it later with the following three api after LibraVM refactor.
    pub fn execute_runtime<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&VMRuntime) -> T,
    {
        self.0.rent(|runtime| f(runtime))
    }

    #[allow(unused)]
    pub fn execute_function<S: ChainState>(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        gas_schedule: &CostTable,
        state_view: &dyn StateView,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
        args: Vec<Value>,
    ) -> VMResult<()> {
        self.0.rent(|runtime| {
            runtime.execute_function(
                state_view,
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
        state_view: &dyn StateView,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
        args: Vec<Value>,
    ) -> VMResult<()> {
        self.0.rent(|runtime| {
            runtime.execute_script(
                state_view,
                chain_state,
                txn_data,
                gas_schedule,
                script,
                args,
            )
        })
    }

    #[allow(unused)]
    pub fn publish_module<S: ChainState>(
        &self,
        module: &[u8],
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
    ) -> VMResult<ModuleId> {
        self.0
            .rent(|runtime| runtime.publish_module(&module, chain_state, txn_data))
    }

    pub(crate) fn load_gas_schedule(
        &self,
        state_view: &dyn StateView,
        data_view: &dyn RemoteCache,
    ) -> VMResult<CostTable> {
        self.0
            .rent(|runtime| runtime.load_gas_schedule(data_view, state_view))
    }
}
