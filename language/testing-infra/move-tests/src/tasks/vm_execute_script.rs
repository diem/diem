// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    language_storage::TypeTag,
    transaction_argument::TransactionArgument,
};
use move_testing_base::{ops::OpGetOutputStream, tasks::Task};
use move_vm_runtime::logging::NoContextLog;
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use std::fmt::Write;

use crate::{
    compiler::{expect_script, move_compile_single_unit},
    ops::{OpGetDefaultSender, OpGetMoveVMSession, OpGetPrecompiledUnit, OpGetPublishedModules},
    types::MoveUnit,
    utils::convert_txn_args,
};

/// Task to execute a Move script in a VM session.
#[derive(Debug)]
pub struct TaskMoveVMExecuteScript {
    script: MoveUnit,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
    senders: Option<Vec<AccountAddress>>,
}

impl<'r, 'l, S> Task<S> for TaskMoveVMExecuteScript
where
    S: OpGetOutputStream
        + OpGetMoveVMSession<'r, 'l>
        + OpGetPrecompiledUnit
        + OpGetDefaultSender
        + OpGetPublishedModules,
{
    fn run(self, mut state: S) -> Result<S> {
        let args = convert_txn_args(&self.args);
        let senders = self
            .senders
            .unwrap_or_else(|| vec![state.get_default_sender()]);

        let unit = match move_compile_single_unit(&mut state, None, self.script)? {
            Some(unit) => unit,
            None => return Ok(state),
        };
        let script = expect_script(unit)?;
        let mut blob = vec![];
        script.serialize(&mut blob)?;

        let sess = state.get_session();

        let log_context = NoContextLog::new();
        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

        match sess.execute_script(
            blob,
            self.ty_args,
            args,
            senders,
            &mut cost_strategy,
            &log_context,
        ) {
            Ok(_) => {
                writeln!(state.get_output_stream(), "Ok")?;
            }
            Err(err) => {
                let output = state.get_output_stream();
                writeln!(output, "Failed to execute script: {:?}", err)?;
            }
        }

        Ok(state)
    }
}

impl TaskMoveVMExecuteScript {
    pub fn new(
        script: MoveUnit,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        senders: Option<Vec<AccountAddress>>,
    ) -> Self {
        Self {
            script,
            ty_args,
            args,
            senders,
        }
    }
}
