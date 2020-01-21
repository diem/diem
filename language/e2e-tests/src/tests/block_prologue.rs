// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::FakeExecutor;
use libra_config::generator;
use libra_crypto::HashValue;
use libra_types::{
    account_config::core_code_address,
    block_metadata::BlockMetadata,
    transaction::{Transaction, TransactionOutput, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use std::collections::btree_map::BTreeMap;

fn is_discarded(output: &TransactionOutput) -> bool {
    match output.status() {
        TransactionStatus::Discard(_) => true,
        _ => false,
    }
}

#[test]
fn execute_block_prologue() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.
    let mut executor = FakeExecutor::from_genesis_file();
    let swarm = generator::validator_swarm_for_testing(10);
    let validator_addr = *swarm.validator_set.iter().nth(0).unwrap().account_address();
    let validator_addr_2 = *swarm.validator_set.iter().nth(1).unwrap().account_address();

    // A proposer proposed a new block. Time should proceed.
    let block1 = BlockMetadata::new(HashValue::zero(), 1, BTreeMap::new(), validator_addr);
    let output = executor
        .execute_transaction_block(vec![Transaction::BlockMetadata(block1)])
        .unwrap();
    assert!(output[0].status() == &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));
    executor.apply_write_set(output[0].write_set());

    // A proposer proposed a block with old time. Should be rejected.
    let block1 = BlockMetadata::new(HashValue::zero(), 0, BTreeMap::new(), validator_addr);
    let output = executor
        .execute_transaction_block(vec![Transaction::BlockMetadata(block1)])
        .unwrap();
    assert!(is_discarded(&output[0]));

    // A nil proposer proposed a new block. Time should remain the same.
    let block1 = BlockMetadata::new(HashValue::zero(), 1, BTreeMap::new(), core_code_address());
    let output = executor
        .execute_transaction_block(vec![Transaction::BlockMetadata(block1)])
        .unwrap();
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    executor.apply_write_set(output[0].write_set());

    // A nil proposer proposed a new block with a different timestamp. Should be rejected.
    let block1 = BlockMetadata::new(HashValue::zero(), 2, BTreeMap::new(), core_code_address());
    let output = executor
        .execute_transaction_block(vec![Transaction::BlockMetadata(block1)])
        .unwrap();
    assert!(is_discarded(&output[0]));

    // A new proposer proposed a new block.
    let block1 = BlockMetadata::new(HashValue::zero(), 2, BTreeMap::new(), validator_addr_2);
    let output = executor
        .execute_transaction_block(vec![Transaction::BlockMetadata(block1)])
        .unwrap();
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
}
