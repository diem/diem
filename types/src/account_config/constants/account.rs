// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag},
};
use once_cell::sync::Lazy;

pub const ACCOUNT_MODULE_NAME: &str = "LibraAccount";

// Account
static ACCOUNT_MODULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraAccount").unwrap());
static ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());
static ACCOUNT_BALANCE_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Balance").unwrap());

/// The ModuleId for the Account module.
pub static ACCOUNT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, ACCOUNT_MODULE_IDENTIFIER.clone()));

// Payment Events
static SENT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("SentPaymentEvent").unwrap());
static RECEIVED_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ReceivedPaymentEvent").unwrap());

pub fn account_balance_struct_name() -> &'static IdentStr {
    &*ACCOUNT_BALANCE_STRUCT_NAME
}

pub fn sent_event_name() -> &'static IdentStr {
    &*SENT_EVENT_NAME
}

pub fn received_event_name() -> &'static IdentStr {
    &*RECEIVED_EVENT_NAME
}

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: ACCOUNT_STRUCT_NAME.to_owned(),
        type_params: vec![],
    }
}

pub fn sent_payment_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: sent_event_name().to_owned(),
        type_params: vec![],
    }
}

pub fn received_payment_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: received_event_name().to_owned(),
        type_params: vec![],
    }
}
