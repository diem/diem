// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules and functions used by Libra System.

use libra_types::{account_config, identifier::Identifier, language_storage::ModuleId};
use once_cell::sync::Lazy;

// Data to resolve basic account and transaction flow functions and structs
/// The ModuleId for the LibraTransactionTimeout module
pub static LIBRA_TRANSACTION_TIMEOUT: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraTransactionTimeout").unwrap(),
    )
});
/// The ModuleId for the Event
pub static EVENT_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("Event").unwrap(),
    )
});
/// The ModuleId for the validator config
pub static VALIDATOR_CONFIG_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("ValidatorConfig").unwrap(),
    )
});
/// The ModuleId for the libra system module
pub static LIBRA_SYSTEM_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraSystem").unwrap(),
    )
});
/// The ModuleId for the gas schedule module
pub static GAS_SCHEDULE_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("GasSchedule").unwrap(),
    )
});

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("create_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
