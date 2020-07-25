// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod command;
pub mod governance;
mod json_rpc;
pub mod operator_key;
pub mod validate_transaction;
pub mod validator_config;
pub mod validator_set;
mod waypoint;

#[cfg(any(test, feature = "testing"))]
pub mod test_helper;

use libra_types::account_address::AccountAddress;

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
