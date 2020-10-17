// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
};
use move_testing_base::{ops::OpGetOutputStream, tasks::Task};
use move_vm_runtime::logging::NoContextLog;
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use std::fmt::Write;

use crate::{
    compiler::{expect_module, move_compile_single_unit},
    ops::{OpGetDefaultSender, OpGetMoveVMSession, OpGetPrecompiledUnit, OpGetPublishedModules},
    types::MoveUnit,
};

/// Task to publish a Move module in a VM session.
#[derive(Debug)]
pub struct TaskMoveVMPublishModule {
    sender: Option<AccountAddress>,
    module: MoveUnit,
}

impl<'r, 'l, S> Task<S> for TaskMoveVMPublishModule
where
    S: OpGetOutputStream
        + OpGetMoveVMSession<'r, 'l>
        + OpGetDefaultSender
        + OpGetPrecompiledUnit
        + OpGetPublishedModules,
{
    fn run(self, mut state: S) -> Result<S> {
        let sender = self.sender.unwrap_or_else(|| state.get_default_sender());

        let unit = match move_compile_single_unit(&mut state, Some(sender), self.module)? {
            Some(unit) => unit,
            None => return Ok(state),
        };
        let module = expect_module(unit)?;
        let mut blob = vec![];
        module.serialize(&mut blob)?;

        let sess = state.get_session();
        let log_context = NoContextLog::new();
        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

        match sess.publish_module(blob, sender, &mut cost_strategy, &log_context) {
            Ok(_) => {
                writeln!(state.get_output_stream(), "Ok")?;
            }
            Err(err) => {
                let output = state.get_output_stream();
                writeln!(output, "Failed to publish module: {:?}", err)?;
            }
        }

        Ok(state)
    }
}

impl TaskMoveVMPublishModule {
    pub fn new(sender: Option<AccountAddress>, module: MoveUnit) -> Self {
        Self { sender, module }
    }
}
