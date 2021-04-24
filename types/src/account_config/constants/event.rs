// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag},
};
use once_cell::sync::Lazy;

pub const EVENT_MODULE_IDENTIFIER: &IdentStr = ident_str!("Event");
pub static EVENT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, EVENT_MODULE_IDENTIFIER.to_owned()));
pub const EVENT_HANDLE_STRUCT_IDENTIFIER: &IdentStr = ident_str!("EventHandle");
pub const EVENT_HANDLE_GENERATOR_STRUCT_IDENTIFIER: &IdentStr = ident_str!("EventHandleGenerator");

pub fn event_handle_generator_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: EVENT_MODULE_IDENTIFIER.to_owned(),
        name: EVENT_HANDLE_GENERATOR_STRUCT_IDENTIFIER.to_owned(),
        type_params: vec![],
    }
}
