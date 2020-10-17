// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Result};
use move_core_types::account_address::AccountAddress;
use move_testing_base::{ops::OpGetOutputStream, tasks::Task};
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;

use crate::{
    compiler::move_compile, ops::OpAddPrecompiledUnit, ops::OpGetDefaultSender, types::MoveSource,
};

/// Describes a task that
///   1. Compiles the specified modules and scripts.
///   2. Stores the compiled units in the global state.
#[derive(Debug)]
pub struct TaskMoveCompile {
    expect_success: bool,
    sender_opt: Option<AccountAddress>,
    targets: BTreeMap<String, MoveSource>,
    dependencies: Vec<MoveSource>,
}

impl<S> Task<S> for TaskMoveCompile
where
    S: OpAddPrecompiledUnit + OpGetOutputStream + OpGetDefaultSender,
{
    fn run(self, mut state: S) -> Result<S> {
        let (vars, targets): (Vec<_>, Vec<_>) = self.targets.into_iter().unzip();

        let sender = self
            .sender_opt
            .unwrap_or_else(|| state.get_default_sender());

        let compiler_res = move_compile(Some(sender), targets, self.dependencies)?;

        match compiler_res {
            Ok(units) => {
                ensure!(vars.len() == units.len(), "Length mismatch");
                for (var, unit) in vars.into_iter().zip(units.into_iter()) {
                    state.add_precompiled_unit(var, unit)?;
                }
            }
            Err(rendered_errors) => {
                if self.expect_success {
                    bail!("Failed to compile modules and scripts");
                } else {
                    write!(state.get_output_stream(), "{}", rendered_errors)?;
                }
            }
        }

        Ok(state)
    }
}

impl TaskMoveCompile {
    pub fn new(
        sender_opt: Option<AccountAddress>,
        targets: BTreeMap<String, MoveSource>,
        dependencies: Vec<MoveSource>,
        expect_success: bool,
    ) -> Self {
        TaskMoveCompile {
            expect_success,
            sender_opt,
            targets,
            dependencies,
        }
    }
}
