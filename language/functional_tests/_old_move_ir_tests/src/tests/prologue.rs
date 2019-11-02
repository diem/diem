// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use language_common::error_codes::*;
use move_ir::{assert_error_type, assert_no_error};

#[test]
fn fail_if_signature_valid_but_pubkey_doesnt_match_auth_key() {
    let mut test_env = TestEnvironment::default();

    let sender_account = test_env.accounts.get_account(0);
    let other_account = test_env.accounts.get_account(1);

    let program = "main() { return; }";
    let sender = 0;
    let sequence_number = test_env.get_txn_sequence_number(sender);
    let max_gas = TestEnvironment::DEFAULT_MAX_GAS;
    let gas_cost = TestEnvironment::DEFAULT_GAS_COST;

    assert!(sender_account.addr != other_account.addr);
    let signed_transaction = test_env.create_signed_txn_with_args(
        to_script(program.as_bytes(), vec![]),
        vec![],
        sender_account.addr,
        other_account, // sender's address, but someone else's account
        sequence_number,
        max_gas,
        gas_cost,
    );

    // creates a transaction whose signature is valid...
    assert!(signed_transaction.verify_signature().is_ok());

    // ...but whose authentication key doesn't match the public key used in the signature
    assert_error_type!(
        test_env.run_txn(signed_transaction),
        ErrorKind::AssertError(EBAD_ACCOUNT_AUTHENTICATION_KEY, _)
    );
}
