// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::{RemoteCache, TransactionDataCache, TransactionEffects},
    runtime::VMRuntime,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_types::{gas_schedule::CostStrategy, values::Value};
use vm::errors::*;

pub struct Session<'r, 'l, R> {
    pub(crate) runtime: &'l VMRuntime,
    pub(crate) data_cache: TransactionDataCache<'r, 'l, R>,
}

impl<'r, 'l, R: RemoteCache> Session<'r, 'l, R> {
    pub fn execute_function(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        _sender: AccountAddress,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        self.runtime.execute_function(
            module,
            function_name,
            ty_args,
            args,
            &mut self.data_cache,
            cost_strategy,
        )
    }

    pub fn execute_script(
        &mut self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        sender: AccountAddress,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        self.runtime.execute_script(
            script,
            ty_args,
            args,
            sender,
            &mut self.data_cache,
            cost_strategy,
        )
    }

    pub fn publish_module(
        &mut self,
        module: Vec<u8>,
        sender: AccountAddress,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        self.runtime
            .publish_module(module, sender, &mut self.data_cache, cost_strategy)
    }

    pub fn finish(self) -> VMResult<TransactionEffects> {
        self.data_cache.into_effects()
    }
}
