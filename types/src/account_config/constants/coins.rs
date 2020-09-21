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

/// Return `Some(struct_name)` if `t` is a `StructTag` representing one of the current Libra coin
/// types (LBR, Coin1, Coin2), `None` otherwise.
pub fn coin_name(t: &TypeTag) -> Option<String> {
    match t {
        TypeTag::Struct(StructTag {
            address,
            module,
            name,
            ..
        }) if *address == CORE_CODE_ADDRESS && module == name => {
            let name_str = name.to_string();
            if name_str == LBR_NAME || name_str == COIN1_NAME || name_str == COIN2_NAME {
                Some(name_str)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[test]
fn coin_names() {
    assert!(coin_name(&coin1_tag()).unwrap() == COIN1_NAME);
    assert!(coin_name(&coin2_tag()).unwrap() == COIN2_NAME);
    assert!(coin_name(&lbr_type_tag()).unwrap() == LBR_NAME);

    assert!(coin_name(&TypeTag::U64) == None);
    let bad_name = Identifier::new("NotACoin").unwrap();
    let bad_coin = TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: bad_name.clone(),
        name: bad_name,
        type_params: vec![],
    });
    assert!(coin_name(&bad_coin) == None);
}
