// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::constants::CORE_CODE_ADDRESS,
    language_storage::{ModuleId, StructTag},
};
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;

static EVENT_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Event").unwrap());
pub static EVENT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, EVENT_MODULE_NAME.clone()));

static EVENT_HANDLE_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("EventHandle").unwrap());
static EVENT_HANDLE_GENERATOR_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("EventHandleGenerator").unwrap());

pub fn event_module_name() -> &'static IdentStr {
    &*EVENT_MODULE_NAME
}

pub fn event_handle_generator_struct_name() -> &'static IdentStr {
    &*EVENT_HANDLE_GENERATOR_STRUCT_NAME
}

pub fn event_handle_struct_name() -> &'static IdentStr {
    &*EVENT_HANDLE_STRUCT_NAME
}

pub fn event_handle_generator_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: event_module_name().to_owned(),
        name: event_handle_generator_struct_name().to_owned(),
        type_params: vec![],
    }
}
