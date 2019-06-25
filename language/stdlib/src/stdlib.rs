// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ir_to_bytecode::parser::{ast::ModuleDefinition, parse_module};
use lazy_static::lazy_static;

macro_rules! make_module_definition {
    ($source_path: literal) => {{
        let struct_body = include_str!($source_path);
        parse_module(struct_body).unwrap()
    }};
}

lazy_static! {
    static ref ACCOUNT_MODULE: ModuleDefinition =
        make_module_definition!("../modules/libra_account.mvir");
    static ref COIN_MODULE: ModuleDefinition =
        make_module_definition!("../modules/libra_coin.mvir");
    static ref NATIVE_HASH_MODULE: ModuleDefinition =
        make_module_definition!("../modules/hash.mvir");
    static ref SIGNATURE_MODULE: ModuleDefinition =
        make_module_definition!("../modules/signature.mvir");
    static ref VALIDATOR_SET_MODULE: ModuleDefinition =
        make_module_definition!("../modules/validator_set.mvir");
}

pub fn account_module() -> ModuleDefinition {
    ACCOUNT_MODULE.clone()
}

pub fn coin_module() -> ModuleDefinition {
    COIN_MODULE.clone()
}

pub fn native_hash_module() -> ModuleDefinition {
    NATIVE_HASH_MODULE.clone()
}

pub fn signature_module() -> ModuleDefinition {
    SIGNATURE_MODULE.clone()
}

pub fn validator_set_module() -> ModuleDefinition {
    VALIDATOR_SET_MODULE.clone()
}
