// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use anyhow::Result;
use move_core_types::{
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use once_cell::sync::Lazy;

pub const LIBRA_MODULE_NAME: &str = "Libra";
static COIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Libra").unwrap());
pub static COIN_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, COIN_MODULE_NAME.clone()));

// TODO: This imposes a few implied restrictions:
//   1) The currency module must be published under the core code address.
//   2) The module name must be the same as the gas specifier.
//   3) The struct name must be the same as the module name
// We need to consider whether we want to switch to a more or fully qualified name.
pub fn type_tag_for_currency_code(currency_code: Identifier) -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: currency_code.clone(),
        name: currency_code,
        type_params: vec![],
    })
}

pub fn from_currency_code_string(currency_code_string: &str) -> Result<Identifier> {
    Identifier::new(currency_code_string)
}
