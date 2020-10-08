// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod secure_backend;
pub mod storage;
pub mod transaction;
pub mod validator_config;

pub mod constants {
    use libra_types::account_config::COIN1_NAME;
    pub const COMMON_NS: &str = "common";
    pub const LAYOUT: &str = "layout";
    pub const VALIDATOR_CONFIG: &str = "validator_config";
    pub const VALIDATOR_OPERATOR: &str = "validator_operator";

    pub const GAS_UNIT_PRICE: u64 = 0;
    pub const MAX_GAS_AMOUNT: u64 = 1_000_000;
    pub const GAS_CURRENCY_CODE: &str = COIN1_NAME;
    pub const TXN_EXPIRATION_SECS: u64 = 3600;
}

#[macro_export]
macro_rules! execute_command {
    ($obj:ident, $command:path, $expected_name:path) => {
        if let $command(cmd) = $obj {
            cmd.execute()
        } else {
            Err(Error::UnexpectedCommand(
                $expected_name.to_string(),
                CommandName::from(&$obj).to_string(),
            ))
        }
    };
}

use libra_crypto::ed25519::Ed25519PublicKey;
use std::{convert::TryInto, fs, path::PathBuf};

pub fn read_key_from_file(path: &PathBuf) -> Result<Ed25519PublicKey, String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    if let Ok(key) = lcs::from_bytes(&data) {
        Ok(key)
    } else {
        let key_data = hex::decode(&data).map_err(|e| e.to_string())?;
        key_data
            .as_slice()
            .try_into()
            .map_err(|e: libra_crypto::CryptoMaterialError| e.to_string())
    }
}
