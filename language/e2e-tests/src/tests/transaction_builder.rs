// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for all of the script encoding functions in language/transaction_builder/lib.rs.
//! Thorough tests that exercise all of the behaviors of the script should live in the language
//! functional tests; these tests are only to ensure that the script encoding functions take the
//! correct types + produce a runnable script.

#![forbid(unsafe_code)]

use crate::{
    account::{Account, AccountData},
    executor::FakeExecutor,
};
use transaction_builder::*;

#[test]
fn register_preburn_burn() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // association account to do the actual burning
    let association = Account::new_association();

    // account to initiate preburning
    let preburner = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

    // Register preburner
    executor.execute_and_apply(preburner.signed_script_txn(encode_register_preburner_script(), 0));
    // Send a preburn request
    executor.execute_and_apply(preburner.signed_script_txn(encode_preburn_script(100), 1));
    // Send a second preburn request
    executor.execute_and_apply(preburner.signed_script_txn(encode_preburn_script(200), 2));

    // Complete the first request by burning
    executor.execute_and_apply(
        association.signed_script_txn(encode_burn_script(*preburner.address()), 1),
    );
    // Complete the second request by cancelling
    executor.execute_and_apply(
        association.signed_script_txn(encode_cancel_burn_script(*preburner.address()), 2),
    );
}
