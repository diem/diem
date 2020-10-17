// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::account_address::AccountAddress;
use move_lang::compiled_unit::CompiledUnit;
use move_testing_base::{ops::OpGetOutputStream, tasks::Task};
use move_vm_runtime::{data_cache::RemoteCache, move_vm::MoveVM, session::Session};
use move_vm_test_utils::convert_txn_effects_to_move_changeset_and_events;
use std::fmt::Write;

use crate::{
    ops::{
        OpApplyChanges, OpGetDefaultSender, OpGetMoveStorage, OpGetMoveVMSession,
        OpGetPrecompiledUnit, OpGetPublishedModules,
    },
    tasks::{
        vm_execute_function::TaskMoveVMExecuteFunction, vm_execute_script::TaskMoveVMExecuteScript,
        vm_publish_module::TaskMoveVMPublishModule,
    },
};

#[derive(Debug)]
pub enum MoveVMSessionTask {
    ExecuteFunction(TaskMoveVMExecuteFunction),
    ExecuteScript(TaskMoveVMExecuteScript),
    PublishModule(TaskMoveVMPublishModule),
}

/// Task to execute Move transactions in a session.
#[derive(Debug)]
pub struct TaskMoveVMExecuteSession {
    tasks: Vec<MoveVMSessionTask>,
}

// The logic here is actually very simple: execute the sub tasks using the session and output.
// It is however unfortunate lifetimes get very complicated...
// TODO: we may be able to simplify this if we make Session a trait instead of a generic type
// with lifetimes.
struct SessionEnv<'r, 'l, 'a, S, E> {
    sess: Session<'r, 'l, S>,
    env: &'a E,
    buffer: String,
}
impl<'r, 'l, 'a, S, E> OpGetOutputStream for SessionEnv<'r, 'l, 'a, S, E> {
    type Output = String;

    fn get_output_stream(&mut self) -> &mut String {
        &mut self.buffer
    }
}
impl<'r, 'l, 'a, S: RemoteCache, E> OpGetMoveVMSession<'r, 'l> for SessionEnv<'r, 'l, 'a, S, E> {
    type Storage = S;

    fn get_session(&mut self) -> &mut Session<'r, 'l, S> {
        &mut self.sess
    }
}
impl<'r, 'l, 'a, S: RemoteCache, E: OpGetDefaultSender> OpGetDefaultSender
    for SessionEnv<'r, 'l, 'a, S, E>
{
    fn get_default_sender(&self) -> AccountAddress {
        self.env.get_default_sender()
    }
}
impl<'r, 'l, 'a, S: RemoteCache, E: OpGetPrecompiledUnit> OpGetPrecompiledUnit
    for SessionEnv<'r, 'l, 'a, S, E>
{
    fn get_precompiled_unit(&self, var: &str) -> Result<CompiledUnit> {
        self.env.get_precompiled_unit(var)
    }
}

impl<'r, 'l, 'a, S: RemoteCache, E: OpGetPublishedModules> OpGetPublishedModules
    for SessionEnv<'r, 'l, 'a, S, E>
{
    fn get_published_modules(&self) -> Vec<Vec<u8>> {
        self.env.get_published_modules()
    }
}

impl<S> Task<S> for TaskMoveVMExecuteSession
where
    S: OpGetOutputStream
        + OpGetMoveStorage
        + OpApplyChanges
        + OpGetPrecompiledUnit
        + OpGetDefaultSender
        + OpGetPublishedModules,
{
    fn run(self, mut state: S) -> Result<S> {
        let vm = MoveVM::new();

        let mut env = SessionEnv {
            sess: vm.new_session(state.get_move_storage()),
            env: &state,
            buffer: String::new(),
        };
        for task in self.tasks {
            match task {
                MoveVMSessionTask::PublishModule(task) => {
                    env = task.run(env)?;
                }
                MoveVMSessionTask::ExecuteFunction(task) => {
                    env = task.run(env)?;
                }
                MoveVMSessionTask::ExecuteScript(task) => {
                    env = task.run(env)?;
                }
            }
        }

        let SessionEnv { sess, buffer, .. } = env;

        let effects = match sess.finish() {
            Ok(effects) => effects,
            Err(err) => bail!("Failed to finish session: {:?}", err),
        };
        let (changes, _events) = convert_txn_effects_to_move_changeset_and_events(effects)?;

        write!(state.get_output_stream(), "{}", buffer)?;
        state.apply_changes(changes)?;

        Ok(state)
    }
}

impl TaskMoveVMExecuteSession {
    pub fn new(tasks: impl IntoIterator<Item = MoveVMSessionTask>) -> Self {
        Self {
            tasks: tasks.into_iter().collect(),
        }
    }
}
