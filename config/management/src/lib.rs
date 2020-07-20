// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_types::account_address::AccountAddress;

pub mod error;
pub mod genesis;
pub mod json_rpc;
pub mod key;
pub mod layout;
pub mod secure_backend;
pub mod storage;
pub mod validator_config;
pub mod validator_operator;
pub mod verify;
pub mod waypoint;

pub mod constants {
    use libra_types::account_config::LBR_NAME;
    pub const COMMON_NS: &str = "common";
    pub const LAYOUT: &str = "layout";
    pub const VALIDATOR_CONFIG: &str = "validator_config";
    pub const VALIDATOR_OPERATOR: &str = "validator_operator";

    pub const GAS_UNIT_PRICE: u64 = 0;
    pub const MAX_GAS_AMOUNT: u64 = 1_000_000;
    pub const GAS_CURRENCY_CODE: &str = LBR_NAME;
    pub const TXN_EXPIRATION_SECS: u64 = 3600;
}

/// Information for validating a transaction after it's been submitted
#[derive(Debug, PartialEq)]
pub struct TransactionContext {
    pub address: AccountAddress,
    pub sequence_number: u64,
}

impl TransactionContext {
    pub fn new(address: AccountAddress, sequence_number: u64) -> TransactionContext {
        TransactionContext {
            address,
            sequence_number,
        }
    }
}
