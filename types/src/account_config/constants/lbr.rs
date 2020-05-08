// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::{
    coin_struct_name, from_currency_code_string, CORE_CODE_ADDRESS,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use once_cell::sync::Lazy;

pub const LBR_NAME: &str = "LBR";

pub static LBR_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(LBR_NAME).unwrap()));
pub static LBR_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

pub fn lbr_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(LBR_NAME).unwrap(),
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}
