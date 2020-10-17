// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
};
use move_testing_base::{ops::OpGetOutputStream, tasks::Task};
use move_vm_runtime::logging::NoContextLog;
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use std::fmt::Write;

use crate::{
    ops::{OpGetDefaultSender, OpGetMoveVMSession},
    utils::convert_txn_args,
};

/// Task to execute a Move function in a VM session.
#[derive(Debug)]
pub struct TaskMoveVMExecuteFunction {
    module_id: ModuleId,
    func_name: Identifier,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
    sender: Option<AccountAddress>,
}

impl<'r, 'l, S> Task<S> for TaskMoveVMExecuteFunction
where
    S: OpGetOutputStream + OpGetMoveVMSession<'r, 'l> + OpGetDefaultSender,
{
    fn run(self, mut state: S) -> Result<S> {
        let args = convert_txn_args(&self.args);
        let sender = self.sender.unwrap_or_else(|| state.get_default_sender());

        let sess = state.get_session();

        let log_context = NoContextLog::new();
        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

        match sess.execute_function(
            &self.module_id,
            &self.func_name,
            self.ty_args,
            args,
            sender,
            &mut cost_strategy,
            &log_context,
        ) {
            Ok(_) => {
                writeln!(state.get_output_stream(), "Ok")?;
            }
            Err(err) => {
                let output = state.get_output_stream();
                writeln!(output, "Failed to execute function: {:?}", err)?;
            }
        }

        Ok(state)
    }
}

impl TaskMoveVMExecuteFunction {
    pub fn new(
        module_id: ModuleId,
        func_name: Identifier,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sender: Option<AccountAddress>,
    ) -> Self {
        Self {
            module_id,
            func_name,
            ty_args,
            args,
            sender,
        }
    }
}
