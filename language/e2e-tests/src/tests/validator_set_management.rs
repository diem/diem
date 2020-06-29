// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::Account,
    common_transactions::{
        add_validator_txn, create_validator_account_txn, create_validator_operator_account_txn,
        reconfigure_txn, set_validator_config_txn, set_validator_operator_txn,
    },
    executor::FakeExecutor,
};
use libra_types::{
    on_chain_config::new_epoch_event_key, transaction::TransactionStatus, vm_status::VMStatus,
};

#[test]
fn validator_add() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root_account = Account::new_libra_root();
    let validator_account = Account::new();

    let txn = create_validator_account_txn(&libra_root_account, &validator_account, 1);
    executor.execute_and_apply(txn);
    executor.new_block();

    let txn = set_validator_config_txn(
        &validator_account,
        &validator_account,
        [
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64,
            0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68,
            0xf7, 0x07, 0x51, 0x1a,
        ]
        .to_vec(),
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
    executor.execute_and_apply(txn);

    let txn = add_validator_txn(&libra_root_account, &validator_account, 2);
    let output = executor.execute_and_apply(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}

#[test]
fn validator_rotate_key_and_reconfigure() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root_account = Account::new_libra_root();
    let validator_account = Account::new();

    let txn = create_validator_account_txn(&libra_root_account, &validator_account, 1);
    executor.execute_and_apply(txn);
    executor.new_block();

    let txn = set_validator_config_txn(
        &validator_account,
        &validator_account,
        [
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64,
            0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68,
            0xf7, 0x07, 0x51, 0x1a,
        ]
        .to_vec(),
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
    let output = executor.execute_and_apply(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );

    let txn = add_validator_txn(&libra_root_account, &validator_account, 2);
    let output = executor.execute_and_apply(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));

    executor.apply_write_set(output.write_set());
    executor.new_block();

    let txn = set_validator_config_txn(
        &validator_account,
        &validator_account,
        [
            0x3d, 0x40, 0x17, 0xc3, 0xe8, 0x43, 0x89, 0x5a, 0x92, 0xb7, 0x0a, 0xa7, 0x4d, 0x1b,
            0x7e, 0xbc, 0x9c, 0x98, 0x2c, 0xcf, 0x2e, 0xc4, 0x96, 0x8c, 0xc0, 0xcd, 0x55, 0xf1,
            0x2a, 0xf4, 0x66, 0x0c,
        ]
        .to_vec(),
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
        &TransactionStatus::Keep(VMStatus::executed())
    );

    let txn = reconfigure_txn(&libra_root_account, 3);
    let output = executor.execute_transaction(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}

#[test]
fn validator_set_operator_set_key_reconfigure() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root_account = Account::new_libra_root();
    let validator_account = Account::new();
    let operator_account = Account::new();

    let txn = create_validator_operator_account_txn(&libra_root_account, &operator_account, 1);
    let output = executor.execute_and_apply(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );

    let txn = create_validator_account_txn(&libra_root_account, &validator_account, 2);
    let output = executor.execute_and_apply(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );
    executor.new_block();

    let txn = set_validator_operator_txn(&validator_account, &operator_account, 0);
    let output = executor.execute_and_apply(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );

    let txn = set_validator_config_txn(
        &operator_account,
        &validator_account,
        [
            0x3d, 0x40, 0x17, 0xc3, 0xe8, 0x43, 0x89, 0x5a, 0x92, 0xb7, 0x0a, 0xa7, 0x4d, 0x1b,
            0x7e, 0xbc, 0x9c, 0x98, 0x2c, 0xcf, 0x2e, 0xc4, 0x96, 0x8c, 0xc0, 0xcd, 0x55, 0xf1,
            0x2a, 0xf4, 0x66, 0x0c,
        ]
        .to_vec(),
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
    let output = executor.execute_and_apply(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );

    let txn = add_validator_txn(&libra_root_account, &validator_account, 3);
    let output = executor.execute_and_apply(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));

    executor.apply_write_set(output.write_set());
    executor.new_block();

    let txn = set_validator_config_txn(
        &operator_account,
        &validator_account,
        [
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64,
            0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68,
            0xf7, 0x07, 0x51, 0x1a,
        ]
        .to_vec(),
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
        &TransactionStatus::Keep(VMStatus::executed())
    );

    let txn = reconfigure_txn(&libra_root_account, 4);
    let output = executor.execute_transaction(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::executed())
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}
