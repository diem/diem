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
/// The ModuleId for the transaction fee module
pub static TRANSACTION_FEE_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("TransactionFee").unwrap(),
    )
});

pub static ASSOCIATION_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        ASSOCIATION_MODULE_NAME.clone(),
    )
});

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("create_unhosted_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BUMP_SEQUENCE_NUMBER_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("bump_sequence_number").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
pub static DISTRIBUTE_TXN_FEES: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("distribute_transaction_fees").unwrap());

static ASSOCIATION_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Association").unwrap());
pub static ASSOCIATION_CAPABILITY_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("PrivilegedCapability").unwrap());
pub static BASE_ASSOCIATION_CAPABILITY_TYPE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("T").unwrap());

// TODO Move this somewhere else
pub fn association_capability_struct_tag() -> StructTag {
    let base_association_cap_tag = StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: ASSOCIATION_MODULE_NAME.to_owned(),
        name: BASE_ASSOCIATION_CAPABILITY_TYPE_NAME.to_owned(),
        type_params: vec![],
    };
    StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: ASSOCIATION_MODULE_NAME.to_owned(),
        name: ASSOCIATION_CAPABILITY_STRUCT_NAME.to_owned(),
        type_params: vec![TypeTag::Struct(base_association_cap_tag)],
    }
}
