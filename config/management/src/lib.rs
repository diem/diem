// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod secure_backend;
pub mod storage;
pub mod validator_config;

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
