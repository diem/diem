// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules and functions used by Libra System.

use libra_types::{account_config, identifier::Identifier, language_storage::ModuleId};
use once_cell::sync::Lazy;

// Data to resolve basic account and transaction flow functions and structs
/// The ModuleId for the Account module
pub static ACCOUNT_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraAccount").unwrap(),
    )
});
/// The ModuleId for the LibraCoin module
pub static COIN_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraCoin").unwrap(),
    )
});
/// The ModuleId for the Event
pub static EVENT_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("Event").unwrap(),
    )
});
/// The ModuleId for the validator config
pub static VALIDATOR_CONFIG_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("ValidatorConfig").unwrap(),
    )
});
/// The ModuleId for the libra system module
pub static LIBRA_SYSTEM_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraSystem").unwrap(),
    )
});
/// The ModuleId for the gas schedule module
pub static GAS_SCHEDULE_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("GasSchedule").unwrap(),
    )
});

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("make").unwrap());
pub static ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());
pub static EMIT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("write_to_event_store").unwrap());
pub static SAVE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("save_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
