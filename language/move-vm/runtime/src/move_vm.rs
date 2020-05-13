// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::VMRuntime;
use bytecode_verifier::VerifiedModule;
use move_core_types::{
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_types::{
    data_store::DataStore, gas_schedule::CostStrategy, transaction_metadata::TransactionMetadata,
    values::Value,
};
use vm::errors::VMResult;

pub struct MoveVM {
    runtime: VMRuntime,
}

impl MoveVM {
    pub fn new() -> Self {
        Self {
            runtime: VMRuntime::new(),
        }
    }

    pub fn execute_function(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        cost_strategy: &mut CostStrategy,
        data_store: &mut dyn DataStore,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        self.runtime.execute_function(
            module,
            function_name,
            ty_args,
            args,
            cost_strategy,
            data_store,
            txn_data,
        )
    }

    pub fn execute_script(
        &self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        cost_strategy: &mut CostStrategy,
        data_store: &mut dyn DataStore,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        self.runtime
            .execute_script(script, ty_args, args, cost_strategy, data_store, txn_data)
    }

    pub fn publish_module(
        &self,
        module: Vec<u8>,
        data_store: &mut dyn DataStore,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        self.runtime
            .publish_module(module, data_store, &txn_data.sender)
    }

    pub fn cache_module(
        &self,
        module: VerifiedModule,
        data_store: &mut dyn DataStore,
    ) -> VMResult<()> {
        self.runtime.cache_module(module, data_store)
    }
}

impl Default for MoveVM {
    fn default() -> Self {
        Self::new()
    }
}
