// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ir_to_bytecode::parser::{ast::ModuleDefinition, parse_module};
use once_cell::sync::Lazy;

macro_rules! make_module_definition {
    ($source_path: literal) => {{
        let struct_body = include_str!($source_path);
        parse_module(struct_body).unwrap()
    }};
}

static ACCOUNT_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/libra_account.mvir"));
static COIN_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/libra_coin.mvir"));
static NATIVE_HASH_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/hash.mvir"));
static SIGNATURE_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/signature.mvir"));
static VALIDATOR_CONFIG_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/validator_config.mvir"));
static LIBRA_TIME_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/libra_time.mvir"));
static LIBRA_TXN_TIMEOUT_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/libra_transaction_timeout.mvir"));
static LIBRA_SYSTEM_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/libra_system.mvir"));
static OFFER_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/offer.mvir"));
static ADDRESS_UTIL_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/address_util.mvir"));
static U64_UTIL_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/u64_util.mvir"));
static VECTOR_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/vector.mvir"));
static BYTEARRAY_UTIL_MODULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/bytearray_util.mvir"));
static GAS_SCHEDULE: Lazy<ModuleDefinition> =
    Lazy::new(|| make_module_definition!("../modules/gas_schedule.mvir"));
static MODULE_DEFS: Lazy<Vec<&'static ModuleDefinition>> = Lazy::new(|| {
    // Note: a module can depend on earlier modules in the list, but not vice versa. Don't try
    // to rearrange without considering this!
    vec![
        &*OFFER_MODULE,
        &*ADDRESS_UTIL_MODULE,
        &*BYTEARRAY_UTIL_MODULE,
        &*COIN_MODULE,
        &*NATIVE_HASH_MODULE,
        &*SIGNATURE_MODULE,
        &*U64_UTIL_MODULE,
        &*VECTOR_MODULE,
        &*VALIDATOR_CONFIG_MODULE,
        &*GAS_SCHEDULE, // depends on Vector
        &*LIBRA_TIME_MODULE,
        &*LIBRA_TXN_TIMEOUT_MODULE, // depends on LibraTimestamp
        &*ACCOUNT_MODULE, // depends on LibraCoin, Event, AddressUtil, BytearrayUtil, U64Util
        &*LIBRA_SYSTEM_MODULE, // depends on LibraAccount, LibraTime, ValidatorConfig
    ]
});

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

pub fn validator_config_module() -> ModuleDefinition {
    VALIDATOR_CONFIG_MODULE.clone()
}

pub fn libra_system_module() -> ModuleDefinition {
    LIBRA_SYSTEM_MODULE.clone()
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

pub fn module_defs() -> &'static [&'static ModuleDefinition] {
    &*MODULE_DEFS
}
