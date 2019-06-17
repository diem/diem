// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use language_common::error_codes::*;
use move_ir::{assert_error_type, assert_no_error};

#[test]
fn cant_run_with_future_sequence_number() {
    let program = b"
main() {
    return;
}";

    // expecting sequence number 0
    assert_error_type!(
        run_with_sequence_number(17, program),
        ErrorKind::AssertError(ESEQUENCE_NUMBER_TOO_NEW, _)
    )
}

#[test]
fn cant_run_with_stale_sequence_number() {
    let mut test_env = TestEnvironment::default();

    let program = "
import 0x0.LibraAccount;
main() {
 let sender;
 let sequence_number;

 sender = get_txn_sender();
 sequence_number = LibraAccount.sequence_number(move(sender));
 assert(move(sequence_number) == 0, 42);

 return;
}";
    assert_no_error!(test_env.run_with_sequence_number(0, to_script(program.as_bytes(), vec![])));

    let program = "
main() {
  return;
}";

    // expecting sequence number 1
    assert_error_type!(
        test_env.run_with_sequence_number(0, to_script(program.as_bytes(), vec![])),
        ErrorKind::AssertError(ESEQUENCE_NUMBER_TOO_OLD, _)
    )
}

#[test]
fn fail_if_cant_pay_deposit() {
    let program = b"
main() {
    let oh_no_ill_never_get_run;
    oh_no_ill_never_get_run = 2;
    return;
}";

    assert_error_type!(
        run_with_max_gas_amount(10_000_000_000_000_000, program),
        ErrorKind::AssertError(ECANT_PAY_GAS_DEPOSIT, _)
    )
}

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
