// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod account;
pub mod command;
mod governance;
mod json_rpc;
mod keys;
mod operator_key;
mod validate_transaction;
mod validator_config;
mod validator_set;
mod waypoint;

#[cfg(any(test, feature = "testing"))]
pub mod test_helper;

use libra_types::account_address::AccountAddress;
use serde::Serialize;

/// Information for validating a transaction after it's been submitted
#[derive(Debug, PartialEq, Serialize)]
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
