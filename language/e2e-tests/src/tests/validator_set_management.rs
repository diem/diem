// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::Account,
    common_transactions::{
        add_validator_txn, create_validator_account_txn, reconfigure_txn, set_validator_config_txn,
    },
    executor::FakeExecutor,
};
use libra_types::{
    on_chain_config::new_epoch_event_key,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn validator_add() {
    let mut executor = FakeExecutor::from_genesis_file();
    let assoc_root_account = Account::new_association();
    let validator_account = Account::new();

    let txn = create_validator_account_txn(&assoc_root_account, &validator_account, 1);
    executor.execute_and_apply(txn);
    executor.new_block();

    let txn = set_validator_config_txn(
        &validator_account,
        vec![255; 32],
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
    executor.execute_and_apply(txn);

    let txn = add_validator_txn(&assoc_root_account, &validator_account, 2);
    let output = executor.execute_and_apply(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}

#[test]
fn validator_rotate_key() {
    let mut executor = FakeExecutor::from_genesis_file();
    let assoc_root_account = Account::new_association();
    let validator_account = Account::new();

    let txn = create_validator_account_txn(&assoc_root_account, &validator_account, 1);
    executor.execute_and_apply(txn);
    executor.new_block();

    let txn = set_validator_config_txn(
        &validator_account,
        vec![255; 32],
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
    executor.execute_and_apply(txn);

    let txn = add_validator_txn(&assoc_root_account, &validator_account, 2);
    let output = executor.execute_and_apply(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));

    executor.apply_write_set(output.write_set());
    executor.new_block();

    let txn = set_validator_config_txn(
        &validator_account,
        vec![251; 32],
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        1,
    );
    let output = executor.execute_transaction(txn);
    executor.apply_write_set(output.write_set());

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );

    let txn = reconfigure_txn(&assoc_root_account, 3);
    let output = executor.execute_transaction(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    println!("{:?}", output);
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}
