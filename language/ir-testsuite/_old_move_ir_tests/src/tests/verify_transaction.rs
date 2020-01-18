// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use libra_types::transaction::{RawTransaction, Script, SignedTransaction, TransactionPayload};
use move_ir::assert_no_error;
use proptest::prelude::*;
use std::time::Duration;

#[test]
fn verify_txn_accepts_good_sequence_number() {
    let test_env = TestEnvironment::default();

    let sequence_number = 0;
    assert_no_error!(test_env.verify_txn_with_context(
        to_script(
            b"
main() {
   let transaction_sequence_number;
   let sender;
   let sequence_number;

   transaction_sequence_number = get_txn_sequence_number();
   assert(copy(transaction_sequence_number) == 0, 42);

   sender = get_txn_sender();
   sequence_number = LibraAccount.sequence_number(move(sender));
   assert(move(sequence_number) == 0, 43);

  return;
}",
            vec![]
        ),
        0,
        sequence_number,
        TestEnvironment::DEFAULT_MAX_GAS,
        TestEnvironment::DEFAULT_GAS_COST,
    ));
}

// TODO: ensure that verify_txn rejects stale sequence numbers, but accepts any sequence number >=
// the current one. this is needed to support the "parking lot" feature of the mempool.
#[test]
fn verify_txn_rejects_bad_sequence_number() {
    let test_env = TestEnvironment::default();

    let sequence_number = 1;
    assert!(test_env
        .verify_txn_with_context(
            to_script(
                b"
main() {
  return;
}",
                vec![]
            ),
            0,
            sequence_number,
            TestEnvironment::DEFAULT_MAX_GAS,
            TestEnvironment::DEFAULT_GAS_COST,
        )
        .is_err());
}

#[test]
fn verify_txn_rejects_bad_signature() {
    let test_env = TestEnvironment::default();

    // Create a transaction signed by account 0 but has the pubkey of account 1.
    let sender_account = test_env.accounts.get_account(0);
    let public_key = test_env.accounts.get_account(1).pubkey;

    let raw_txn = RawTransaction::new_script(
        sender_account.addr,
        0,
        Script::new(to_script(b"main() { return; }", vec![]), vec![]),
        "".to_string(),
        TestEnvironment::DEFAULT_MAX_GAS,
        TestEnvironment::DEFAULT_GAS_COST,
        Duration::from_secs(u64::max_value()),
    );

    let signed_txn = raw_txn
        .clone()
        .sign(&sender_account.privkey, &public_key)
        .unwrap();

    let signed_txn_with_bad_pubkey =
        SignedTransaction::new_for_test(raw_txn, public_key, signed_txn.signature());
    assert!(test_env.verify_txn(signed_txn_with_bad_pubkey).is_err());
}

#[test]
fn verify_txn_accepts_good_signature() {
    let test_env = TestEnvironment::default();

    let sender_account = test_env.accounts.get_account(0);
    let signed_txn = test_env.create_user_txn(
        to_script(b"main() { return; }", vec![]),
        sender_account.addr,
        sender_account,
        0,
        TestEnvironment::DEFAULT_MAX_GAS,
        TestEnvironment::DEFAULT_GAS_COST,
    );
    assert!(test_env.verify_txn(signed_txn).is_ok());
}

#[test]
fn verify_txn_rejects_write_set() {
    let test_env = TestEnvironment::default();
    assert_ne!(test_env.get_version(), 0);

    proptest!(|(txn in SignedTransaction::write_set_strategy())| {
        test_env.verify_txn(txn).expect_err("non-genesis write set txns should fail verification");
    });
}

#[test]
fn verify_txn_rejects_genesis_deletion() {
    let test_env = TestEnvironment::empty();
    assert_eq!(test_env.get_version(), 0);

    proptest!(|(txn in SignedTransaction::write_set_strategy())| {
        let write_set = match txn.payload() {
            TransactionPayload::WriteSet(write_set) => write_set,
            TransactionPayload::Program | TransactionPayload::Script(_) | TransactionPayload::Module(_) => panic!(
                "write_set_strategy shouldn't generate other transactions",
            ),
        };
        let any_deletions = write_set.iter().any(|(_, write_op)| write_op.is_deletion());
        if any_deletions {
            test_env.verify_txn(txn).expect_err("genesis write set with deletes should be rejected");
        } else {
            test_env.verify_txn(txn).expect("genesis write set txns should verify correctly");
        }
    });
}

#[test]
fn verify_txn_accepts_genesis_write_set() {
    let test_env = TestEnvironment::empty();
    assert_eq!(test_env.get_version(), 0);

    proptest!(|(txn in SignedTransaction::genesis_strategy())| {
        test_env.verify_txn(txn).expect("genesis write set txns should verify correctly");
    });
}
