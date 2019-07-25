// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{assert_prologue_parity, executor::FakeExecutor};
use assert_matches::assert_matches;
use crypto::signing::KeyPair;
use types::{
    access_path::AccessPath,
    account_config,
    test_helpers::transaction_test_helpers,
    transaction::TransactionStatus,
    vm_error::{VMStatus, VMValidationStatus},
    write_set::{WriteOp, WriteSetMut},
};

#[test]
fn invalid_genesis_write_set() {
    let executor = FakeExecutor::no_genesis();
    // Genesis write sets are not allowed to contain deletions.
    let write_op = (AccessPath::default(), WriteOp::Deletion);
    let write_set = WriteSetMut::new(vec![write_op]).freeze().unwrap();
    let address = account_config::association_address();
    let keypair = KeyPair::new(::crypto::signing::generate_keypair().0);
    let signed_txn = transaction_test_helpers::get_write_set_txn(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(write_set),
    )
    .into_inner();
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()),
        executor.execute_transaction(signed_txn).status(),
        VMStatus::Validation(VMValidationStatus::InvalidWriteSet)
    );
}
