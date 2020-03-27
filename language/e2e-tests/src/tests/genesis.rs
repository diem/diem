// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    assert_prologue_parity, assert_status_eq, data_store::GENESIS_WRITE_SET,
    executor::FakeExecutor, keygen::KeyGen, transaction_status_eq,
};
use libra_crypto::ed25519::*;
use libra_types::{
    access_path::AccessPath,
    account_config,
    test_helpers::transaction_test_helpers,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSetMut},
};

#[test]
fn invalid_genesis_write_set() {
    let executor = FakeExecutor::no_genesis();
    // Genesis write sets are not allowed to contain deletions.
    let write_op = (AccessPath::default(), WriteOp::Deletion);
    let write_set = WriteSetMut::new(vec![write_op]).freeze().unwrap();
    let address = account_config::association_address();
    let (private_key, public_key) = compat::generate_keypair(None);
    let txn = transaction_test_helpers::get_write_set_txn(
        address,
        0,
        &private_key,
        public_key,
        Some(write_set),
    )
    .into_inner();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::INVALID_WRITE_SET)
    );
}

#[test]
fn execute_genesis_write_set() {
    let executor = FakeExecutor::no_genesis();
    let write_set = GENESIS_WRITE_SET.clone();
    let address = account_config::association_address();
    let mut keygen = KeyGen::from_seed([9u8; 32]);

    let (private_key, public_key) = keygen.generate_keypair();
    let txn = transaction_test_helpers::get_write_set_txn(
        address,
        0,
        &private_key,
        public_key,
        Some(write_set),
    )
    .into_inner();

    // Executing the genesis transaction should success.
    assert!(!executor.execute_transaction(txn).status().is_discarded());
}
