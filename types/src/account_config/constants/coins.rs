// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::{
    coin_struct_name, from_currency_code_string, CORE_CODE_ADDRESS,
};
use move_core_types::language_storage::{StructTag, TypeTag};

pub const COIN1_NAME: &str = "Coin1";
pub const COIN2_NAME: &str = "Coin2";

pub fn coin1_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(COIN1_NAME).unwrap(),
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}

pub fn coin2_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(COIN2_NAME).unwrap(),
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}
