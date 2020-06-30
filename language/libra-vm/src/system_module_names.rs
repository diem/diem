// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules, functions, and types used by Libra System.

use libra_types::account_config;
use move_core_types::{
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
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
pub static LIBRA_CONFIG_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraConfig").unwrap(),
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
    Lazy::new(|| Identifier::new("create_unhosted_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static WRITESET_EPILOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static SUCCESS_EPILOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("success_epilogue").unwrap());
pub static FAILURE_EPILOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("failure_epilogue").unwrap());
pub static BUMP_SEQUENCE_NUMBER_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("bump_sequence_number").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
pub static DISTRIBUTE_TXN_FEES: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("distribute_transaction_fees").unwrap());

static ROLES_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Roles").unwrap());
pub static ROLES_PRIVILEGE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Privilege").unwrap());
pub static MODULE_PUBLISHING_TYPE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("PublishModule").unwrap());

pub fn module_publishing_capability_struct_tag() -> StructTag {
    let mod_publishing_tag = StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: account_config::ACCOUNT_MODULE_IDENTIFIER.to_owned(),
        name: MODULE_PUBLISHING_TYPE_NAME.to_owned(),
        type_params: vec![],
    };
    StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: ROLES_MODULE_NAME.to_owned(),
        name: ROLES_PRIVILEGE_NAME.to_owned(),
        type_params: vec![TypeTag::Struct(mod_publishing_tag)],
    }
}
