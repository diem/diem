// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules, functions, and types used by Libra System.

use libra_types::account_config;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use once_cell::sync::Lazy;

// Data to resolve basic account and transaction flow functions and structs
/// The ModuleId for the libra writeset manager module
pub static LIBRA_WRITESET_MANAGER_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraWriteSetManager").unwrap(),
    )
});

/// The ModuleId for the libra block module
pub static LIBRA_BLOCK_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraBlock").unwrap(),
    )
});

// Names for special functions and structs
pub static SCRIPT_PROLOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("script_prologue").unwrap());
pub static MODULE_PROLOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("module_prologue").unwrap());
pub static WRITESET_PROLOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("prologue").unwrap());
pub static WRITESET_EPILOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static USER_EPILOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BUMP_SEQUENCE_NUMBER_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("bump_sequence_number").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
