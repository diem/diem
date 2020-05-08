// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::VMRuntime;
use bytecode_verifier::VerifiedModule;
use libra_types::language_storage::{ModuleId, TypeTag};
use move_core_types::{gas_schedule::CostTable, identifier::IdentStr};
use move_vm_types::{
    chain_state::ChainState, transaction_metadata::TransactionMetadata, values::Value,
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

    pub fn execute_function<S: ChainState>(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        gas_schedule: &CostTable,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        self.runtime.execute_function(
            chain_state,
            txn_data,
            gas_schedule,
            module,
            function_name,
            ty_args,
            args,
        )
    }

    pub fn execute_script<S: ChainState>(
        &self,
        script: Vec<u8>,
        gas_schedule: &CostTable,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        self.runtime
            .execute_script(chain_state, txn_data, gas_schedule, script, ty_args, args)
    }

    pub fn publish_module<S: ChainState>(
        &self,
        module: Vec<u8>,
        chain_state: &mut S,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        self.runtime.publish_module(module, chain_state, txn_data)
    }

    pub fn cache_module<S: ChainState>(
        &self,
        module: VerifiedModule,
        chain_state: &mut S,
    ) -> VMResult<()> {
        self.runtime.cache_module(module, chain_state)
    }
}

impl Default for MoveVM {
    fn default() -> Self {
        Self::new()
    }
}
