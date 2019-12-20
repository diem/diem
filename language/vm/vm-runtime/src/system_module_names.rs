// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules and functions used by Libra System.

use libra_types::{account_config, identifier::Identifier, language_storage::ModuleId};

// Data to resolve basic account and transaction flow functions and structs
lazy_static! {
    /// The ModuleId for the Account module
    pub static ref ACCOUNT_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraAccount").unwrap()) };
    /// The ModuleId for the LibraCoin module
    pub static ref COIN_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraCoin").unwrap()) };
    /// The ModuleId for the Event
    pub static ref EVENT_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("Event").unwrap()) };
    /// The ModuleId for the validator config
    pub static ref VALIDATOR_CONFIG_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("ValidatorConfig").unwrap()) };
    /// The ModuleId for the libra system module
    pub static ref LIBRA_SYSTEM_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraSystem").unwrap()) };
    /// The ModuleId for the gas schedule module
    pub static ref GAS_SCHEDULE_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("GasSchedule").unwrap()) };
}

// Names for special functions and structs
lazy_static! {
    pub static ref CREATE_ACCOUNT_NAME: Identifier = Identifier::new("make").unwrap();
    pub static ref ACCOUNT_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
    pub static ref EMIT_EVENT_NAME: Identifier = Identifier::new("write_to_event_store").unwrap();
    pub static ref SAVE_ACCOUNT_NAME: Identifier = Identifier::new("save_account").unwrap();
    pub static ref PROLOGUE_NAME: Identifier = Identifier::new("prologue").unwrap();
    pub static ref EPILOGUE_NAME: Identifier = Identifier::new("epilogue").unwrap();
}
