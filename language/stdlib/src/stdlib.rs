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
    static ref BLOCK_MODULE: ModuleDefinition =
        make_module_definition!("../modules/block.mvir");
    static ref COIN_MODULE: ModuleDefinition =
        make_module_definition!("../modules/libra_coin.mvir");
    static ref NATIVE_HASH_MODULE: ModuleDefinition =
        make_module_definition!("../modules/hash.mvir");
    static ref SIGNATURE_MODULE: ModuleDefinition =
        make_module_definition!("../modules/signature.mvir");
    static ref VALIDATOR_SET_MODULE: ModuleDefinition =
        make_module_definition!("../modules/validator_set.mvir");
    static ref ADDRESS_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/address_util.mvir");
    static ref U64_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/u64_util.mvir");
    static ref BYTEARRAY_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/bytearray_util.mvir");
    static ref EVENT_MODULE: ModuleDefinition = make_module_definition!("../modules/event.mvir");
    static ref MODULE_DEFS: Vec<&'static ModuleDefinition> = {
        // Note: a module can depend on earlier modules in the list, but not vice versa. Don't try
        // to rearrange without considering this!
        vec![
            &*ADDRESS_UTIL_MODULE,
            &*BLOCK_MODULE,
            &*BYTEARRAY_UTIL_MODULE,
            &*COIN_MODULE,
            &*NATIVE_HASH_MODULE,
            &*SIGNATURE_MODULE,
            &*U64_UTIL_MODULE,
            &*EVENT_MODULE, // depends on AddressUtil, BytearrayUtil, Hash, U64Util
            &*ACCOUNT_MODULE, // depends on Coin, Event, AddressUtil, BytearrayUtil, U64Util
            &*VALIDATOR_SET_MODULE // will eventually depend on Account, others
        ]
    };
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

pub fn address_util_module() -> ModuleDefinition {
    ADDRESS_UTIL_MODULE.clone()
}

pub fn u64_util_module() -> ModuleDefinition {
    U64_UTIL_MODULE.clone()
}

pub fn bytearray_util_module() -> ModuleDefinition {
    BYTEARRAY_UTIL_MODULE.clone()
}

pub fn event_module() -> ModuleDefinition {
    EVENT_MODULE.clone()
}

pub fn module_defs() -> &'static [&'static ModuleDefinition] {
    &*MODULE_DEFS
}
