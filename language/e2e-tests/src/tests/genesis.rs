// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{data_store::GENESIS_CHANGE_SET, executor::FakeExecutor};
use libra_types::transaction::Transaction;

#[test]
fn execute_genesis_write_set() {
    let executor = FakeExecutor::no_genesis();
    let txn = Transaction::WaypointWriteSet(GENESIS_CHANGE_SET.clone());
    let mut output = executor.execute_transaction_block(vec![txn]).unwrap();

    // Executing the genesis transaction should succeed
    assert_eq!(output.len(), 1);
    assert!(!output.pop().unwrap().status().is_discarded())
}
