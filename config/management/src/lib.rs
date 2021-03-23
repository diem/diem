// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod secure_backend;
pub mod storage;
pub mod transaction;
pub mod validator_config;
pub mod waypoint;

pub mod constants {
    use diem_types::account_config::XUS_NAME;
    pub const COMMON_NS: &str = "common";
    pub const LAYOUT: &str = "layout";
    pub const VALIDATOR_CONFIG: &str = "validator_config";
    pub const VALIDATOR_OPERATOR: &str = "validator_operator";

    pub const GAS_UNIT_PRICE: u64 = 0;
    pub const MAX_GAS_AMOUNT: u64 = 1_000_000;
    pub const GAS_CURRENCY_CODE: &str = XUS_NAME;
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

use diem_crypto::ed25519::Ed25519PublicKey;
use std::{convert::TryInto, fs, path::Path};

/// Reads a given ed25519 public key from file. Attempts to read the key using
/// bcs encoding first. If this fails, attempts reading the key using hex.
pub fn read_key_from_file(path: &Path) -> Result<Ed25519PublicKey, String> {
    let bcs_bytes = fs::read(path).map_err(|e| e.to_string())?;
    if let Ok(key) = bcs::from_bytes(&bcs_bytes) {
        Ok(key)
    } else {
        let hex_bytes = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let hex_bytes = hex_bytes.trim(); // Remove leading or trailing whitespace
        let key_data = hex::decode(&hex_bytes).map_err(|e| e.to_string())?;
        key_data
            .as_slice()
            .try_into()
            .map_err(|e: diem_crypto::CryptoMaterialError| e.to_string())
    }
}
