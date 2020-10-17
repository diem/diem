// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_lang::compiled_unit::CompiledUnit;
use move_vm_runtime::{data_cache::RemoteCache, session::Session};
use move_vm_test_utils::ChangeSet;

pub trait OpAddPrecompiledUnit {
    fn add_precompiled_unit(&mut self, var: String, unit: CompiledUnit) -> Result<()>;
}

pub trait OpGetPrecompiledUnit {
    fn get_precompiled_unit(&self, var: &str) -> Result<CompiledUnit>;
}

pub trait OpGetMoveStorage {
    type Storage: RemoteCache;

    fn get_move_storage(&self) -> &Self::Storage;
}
pub trait OpApplyChanges {
    fn apply_changes(&mut self, changes: ChangeSet) -> Result<()>;
}

pub trait OpGetMoveVMSession<'r, 'l> {
    type Storage: RemoteCache + 'r;

    fn get_session(&mut self) -> &mut Session<'r, 'l, Self::Storage>;
}

pub trait OpGetDefaultSender {
    fn get_default_sender(&self) -> AccountAddress;
}

pub trait OpGetPublishedModules {
    fn get_published_modules(&self) -> Vec<Vec<u8>>;
}
