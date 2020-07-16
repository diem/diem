// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountData, common_transactions::peer_to_peer_txn, data_store::GENESIS_CHANGE_SET,
    executor::FakeExecutor,
};
use libra_types::transaction::{Transaction, TransactionStatus};

#[test]
fn no_deletion_in_genesis() {
    let genesis = GENESIS_CHANGE_SET.clone();
    assert!(!genesis.write_set().iter().any(|(_, op)| op.is_deletion()))
}

#[test]
fn execute_genesis_write_set() {
    let executor = FakeExecutor::no_genesis();
    let txn = Transaction::WaypointWriteSet(GENESIS_CHANGE_SET.clone());
    let mut output = executor.execute_transaction_block(vec![txn]).unwrap();

    // Executing the genesis transaction should succeed
    assert_eq!(output.len(), 1);
    assert!(!output.pop().unwrap().status().is_discarded())
}

#[test]
fn execute_genesis_and_drop_other_transaction() {
    let executor = FakeExecutor::no_genesis();
    let txn = Transaction::WaypointWriteSet(GENESIS_CHANGE_SET.clone());

    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    let txn2 = peer_to_peer_txn(&sender.account(), &receiver.account(), 11, 1000);

    let mut output = executor
        .execute_transaction_block(vec![txn, Transaction::UserTransaction(txn2)])
        .unwrap();

    // Transaction that comes after genesis should be dropped.
    assert_eq!(output.len(), 2);
    assert_eq!(output.pop().unwrap().status(), &TransactionStatus::Retry)
}
