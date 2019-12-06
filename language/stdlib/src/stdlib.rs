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
    static ref VALIDATOR_CONFIG_MODULE: ModuleDefinition =
        make_module_definition!("../modules/validator_config.mvir");
    static ref LIBRA_SYSTEM_MODULE: ModuleDefinition =
        make_module_definition!("../modules/libra_system.mvir");
    static ref ADDRESS_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/address_util.mvir");
    static ref U64_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/u64_util.mvir");
    static ref VECTOR_MODULE: ModuleDefinition =
        make_module_definition!("../modules/vector.mvir");
    static ref BYTEARRAY_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/bytearray_util.mvir");
    static ref TRANSACTION_FEE_DISTRIBUTION_MODULE: ModuleDefinition =
        make_module_definition!("../modules/transaction_fee_distribution.mvir");
    static ref GAS_SCHEDULE: ModuleDefinition = make_module_definition!("../modules/gas_schedule.mvir");
    static ref CHANNEL_TXN_MODULE: ModuleDefinition =
        make_module_definition!("../modules/channel_transaction.mvir");
    static ref CHANNEL_UTIL_MODULE: ModuleDefinition =
        make_module_definition!("../modules/channel_util.mvir");
    static ref HASH_TIME_LOCK_MODULE: ModuleDefinition =
        make_module_definition!("../modules/hash_time_lock.mvir");
    static ref CHANNEL_SCRIPT_MODULE: ModuleDefinition =
        make_module_definition!("../modules/channel_script.mvir");
    static ref CONSENSUS_CONF_MODULE: ModuleDefinition = make_module_definition!("../modules/consensus_config.mvir");
    static ref MODULE_DEFS: Vec<&'static ModuleDefinition> = {
        // Note: a module can depend on earlier modules in the list, but not vice versa. Don't try
        // to rearrange without considering this!
        vec![
            &*ADDRESS_UTIL_MODULE,
            &*BYTEARRAY_UTIL_MODULE,
            &*COIN_MODULE,
            &*NATIVE_HASH_MODULE,
            &*SIGNATURE_MODULE,
            &*U64_UTIL_MODULE,
            &*VECTOR_MODULE,
            &*VALIDATOR_CONFIG_MODULE,
            &*GAS_SCHEDULE, // depends on Vector
            &*CHANNEL_TXN_MODULE,
            &*CHANNEL_UTIL_MODULE,
            &*ACCOUNT_MODULE, // depends on LibraCoin, Event, AddressUtil, BytearrayUtil, U64Util, ChannelTransaction
            &*CONSENSUS_CONF_MODULE, // depends on LibraAccount, Vector
            &*LIBRA_SYSTEM_MODULE, // depends on LibraAccount, ValidatorConfig
            &*TRANSACTION_FEE_DISTRIBUTION_MODULE, // depends on Block, ValidatorSet, LibraCoin, LibraAccount,
            &*HASH_TIME_LOCK_MODULE,
            &*CHANNEL_SCRIPT_MODULE,
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

pub fn channel_script_module() -> ModuleDefinition {
    CHANNEL_SCRIPT_MODULE.clone()
}

pub fn channel_txn_module() -> ModuleDefinition {
    CHANNEL_TXN_MODULE.clone()
}

pub fn channel_util_module() -> ModuleDefinition {
    CHANNEL_UTIL_MODULE.clone()
}

pub fn hash_time_lock_module() -> ModuleDefinition {
    HASH_TIME_LOCK_MODULE.clone()
}
