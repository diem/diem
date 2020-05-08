// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use once_cell::sync::Lazy;

static ACCOUNT_LIMITS_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("AccountLimits").unwrap());
pub static ACCOUNT_LIMITS_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, ACCOUNT_LIMITS_MODULE_NAME.clone()));
pub static ACCOUNT_LIMITS_WINDOW_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Window").unwrap());

pub fn account_limits_module_name() -> &'static IdentStr {
    &*ACCOUNT_LIMITS_MODULE_NAME
}

pub fn account_limits_window_struct_name() -> &'static IdentStr {
    &*ACCOUNT_LIMITS_WINDOW_STRUCT_NAME
}
