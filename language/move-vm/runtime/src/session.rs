// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::{RemoteCache, TransactionDataCache, TransactionEffects},
    logging::LogContext,
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
    /// Execute a Move function with the given arguments. This is mainly designed for an external environment
    /// to invoke system logic written in Move.
    ///
    /// The caller MUST ensure
    ///   - The function to be called and the module containing it exist.
    ///   - All types and modules referred to by the type arguments exist.
    ///   - All arguments are valid and match the signature of the function called.
    ///
    /// The Move VM MUST return an invariant violation if the caller fails to follow any of the rules above.
    ///
    /// Currently if any other error occurs during execution, the Move VM will simply propagate that error back
    /// to the outer environment without handling/translating it. This behavior may be revised in the future.
    ///
    /// In case an invariant violation occurs, the whole Session should be considered corrupted and one shall
    /// not proceed with effect generation.
    pub fn execute_function(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        _sender: AccountAddress,
        cost_strategy: &mut CostStrategy,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        self.runtime.execute_function(
            module,
            function_name,
            ty_args,
            args,
            &mut self.data_cache,
            cost_strategy,
            log_context,
        )
    }

    /// Execute a transaction script.
    ///
    /// The Move VM MUST return a user error (in other words, an error that's not an invariant violation) if
    ///   - The script fails to deserialize or verify.
    ///   - Type arguments refer to a non-existent type.
    ///   - Arguments (senders included) are invalid or fail to match the signature of the script.
    ///
    /// If any other error occurs during execution, the Move VM MUST propagate that error back to the caller.
    /// Besides, no user input should cause the Move VM to return an invariant violation.
    ///
    /// In case an invariant violation occurs, the whole Session should be considered corrupted and one shall
    /// not proceed with effect generation.
    pub fn execute_script(
        &mut self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        senders: Vec<AccountAddress>,
        cost_strategy: &mut CostStrategy,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        self.runtime.execute_script(
            script,
            ty_args,
            args,
            senders,
            &mut self.data_cache,
            cost_strategy,
            log_context,
        )
    }

    /// Publish the given module.
    ///
    /// The Move VM MUST return a user error (in other words, an error that's not an invariant violation) if
    ///   - The module fails to deserialize or verify.
    ///   - A module with the same ModuleId already exists in the environment.
    ///   - The sender address does not match that of the module.
    ///
    /// The Move VM should not be able to produce other user errors.
    /// Besides, no user input should cause the Move VM to return an invariant violation.
    ///
    /// In case an invariant violation occurs, the whole Session should be considered corrupted and one shall
    /// not proceed with effect generation.
    pub fn publish_module(
        &mut self,
        module: Vec<u8>,
        sender: AccountAddress,
        cost_strategy: &mut CostStrategy,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        self.runtime.publish_module(
            module,
            sender,
            false,
            &mut self.data_cache,
            cost_strategy,
            log_context,
        )
    }

    // TODO: merge with publish_module?
    pub fn publish_module_ext(
        &mut self,
        module: Vec<u8>,
        sender: AccountAddress,
        allow_republish: bool,
        cost_strategy: &mut CostStrategy,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        self.runtime.publish_module(
            module,
            sender,
            allow_republish,
            &mut self.data_cache,
            cost_strategy,
            log_context,
        )
    }

    pub fn num_mutated_accounts(&self, sender: &AccountAddress) -> u64 {
        self.data_cache.num_mutated_accounts(sender)
    }

    /// Finish up the session and produce the side effects.
    ///
    /// This function should always succeed with no user errors returned, barring invariant violations.
    ///
    /// This MUST NOT be called if there is a previous invocation that failed with an invariant violation.
    pub fn finish(self) -> VMResult<TransactionEffects> {
        self.data_cache
            .into_effects()
            .map_err(|e| e.finish(Location::Undefined))
    }
}
