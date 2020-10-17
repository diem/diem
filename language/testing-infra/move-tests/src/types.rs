// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{format_err, Result};
use move_core_types::account_address::AccountAddress;
use move_lang::compiled_unit::CompiledUnit;
use move_testing_base::ops::OpGetOutputStream;
use move_testing_base::Task;
use move_vm_test_utils::{ChangeSet, InMemoryStorage};
use std::collections::BTreeMap;
use std::convert::From;
use std::default::Default;

use crate::ops::{
    OpAddPrecompiledUnit, OpApplyChanges, OpGetDefaultSender, OpGetMoveStorage,
    OpGetPrecompiledUnit, OpGetPublishedModules,
};
use crate::tasks::{TaskMoveCompile, TaskMoveVMExecuteSession, TaskShowMoveStorage};

#[derive(Debug)]
pub struct MoveTestingEnv {
    pub output: String,
    pub storage: InMemoryStorage,
    pub compiled_units: BTreeMap<String, CompiledUnit>,
}

impl OpGetMoveStorage for MoveTestingEnv {
    type Storage = InMemoryStorage;

    fn get_move_storage(&self) -> &InMemoryStorage {
        &self.storage
    }
}

impl OpApplyChanges for MoveTestingEnv {
    fn apply_changes(&mut self, changeset: ChangeSet) -> Result<()> {
        self.storage.apply(changeset)
    }
}

impl OpAddPrecompiledUnit for MoveTestingEnv {
    fn add_precompiled_unit(&mut self, var: String, unit: CompiledUnit) -> Result<()> {
        self.compiled_units.insert(var, unit);
        Ok(())
    }
}

impl OpGetOutputStream for MoveTestingEnv {
    type Output = String;

    fn get_output_stream(&mut self) -> &mut String {
        &mut self.output
    }
}

impl OpGetDefaultSender for MoveTestingEnv {
    fn get_default_sender(&self) -> AccountAddress {
        AccountAddress::from_hex_literal("0x1000").unwrap()
    }
}

impl OpGetPrecompiledUnit for MoveTestingEnv {
    fn get_precompiled_unit(&self, var: &str) -> Result<CompiledUnit> {
        self.compiled_units
            .get(var)
            .cloned()
            .ok_or_else(|| format_err!("Failed to fetch pre-compiled unit {}", var))
    }
}

impl OpGetPublishedModules for MoveTestingEnv {
    fn get_published_modules(&self) -> Vec<Vec<u8>> {
        self.storage.modules().cloned().collect()
    }
}

impl MoveTestingEnv {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            storage: InMemoryStorage::new(),
            compiled_units: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum MoveSource {
    Embedded(String),
    Path(String),
}

#[derive(Debug)]
pub enum MoveUnit {
    Source {
        source: MoveSource,
        deps: Option<Vec<MoveSource>>,
    },
    Precompiled(String),
}

#[derive(Debug)]
pub struct Config {
    pub genesis: GenesisOption,
}

#[derive(Debug)]
pub enum GenesisOption {
    Std,
    None,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            genesis: GenesisOption::Std,
        }
    }
}

#[derive(Debug)]
pub enum MoveTask {
    MoveCompile(TaskMoveCompile),
    VMExecuteSession(TaskMoveVMExecuteSession),
    ShowMoveStorage(TaskShowMoveStorage),
}

#[derive(Debug)]
pub struct MoveTestingFlow {
    pub config: Config,
    pub tasks: Vec<MoveTask>,
}

impl From<TaskMoveCompile> for MoveTask {
    fn from(task: TaskMoveCompile) -> Self {
        MoveTask::MoveCompile(task)
    }
}

impl From<TaskMoveVMExecuteSession> for MoveTask {
    fn from(task: TaskMoveVMExecuteSession) -> Self {
        MoveTask::VMExecuteSession(task)
    }
}

impl From<TaskShowMoveStorage> for MoveTask {
    fn from(task: TaskShowMoveStorage) -> Self {
        MoveTask::ShowMoveStorage(task)
    }
}

impl Task<MoveTestingEnv> for MoveTask {
    fn run(self, state: MoveTestingEnv) -> Result<MoveTestingEnv> {
        match self {
            Self::MoveCompile(task) => task.run(state),
            Self::VMExecuteSession(task) => task.run(state),
            Self::ShowMoveStorage(task) => task.run(state),
        }
    }
}
