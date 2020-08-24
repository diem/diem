// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::{from_currency_code_string, CORE_CODE_ADDRESS};
use move_core_types::{
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use once_cell::sync::Lazy;

pub const LBR_NAME: &str = "LBR";
pub const COIN1_NAME: &str = "Coin1";
pub const COIN2_NAME: &str = "Coin2";

pub fn coin1_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(COIN1_NAME).unwrap(),
        name: from_currency_code_string(COIN1_NAME).unwrap(),
        type_params: vec![],
    })
}

pub fn coin2_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(COIN2_NAME).unwrap(),
        name: from_currency_code_string(COIN2_NAME).unwrap(),
        type_params: vec![],
    })
}

pub static LBR_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(LBR_NAME).unwrap()));
pub static LBR_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new(LBR_NAME).unwrap());

pub fn lbr_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(LBR_NAME).unwrap(),
        name: from_currency_code_string(LBR_NAME).unwrap(),
        type_params: vec![],
    })
}
