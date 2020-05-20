// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::Account,
    common_transactions::{
        create_validator_account_txn, register_validator_txn, rotate_consensus_pubkey_txn,
    },
    executor::FakeExecutor,
};
use libra_types::{
    on_chain_config::new_epoch_event_key,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};
use transaction_builder::*;

#[test]
fn validator_add() {
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();
    let validator_account = Account::new();

    let txn = create_validator_account_txn(&genesis_account, &validator_account, 1);
    executor.execute_and_apply(txn);

    let mint_amount = 10_000_000;
    executor.execute_and_apply(genesis_account.signed_script_txn(
        encode_mint_lbr_to_address_script(&validator_account.address(), vec![], mint_amount),
        2,
    ));
    executor.new_block();

    let txn = register_validator_txn(
        &validator_account,
        vec![255; 32],
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
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
    let genesis_account = Account::new_association();
    let validator_account = Account::new();

    let txn = create_validator_account_txn(&genesis_account, &validator_account, 1);
    executor.execute_and_apply(txn);

    let mint_amount = 10_000_000;
    executor.execute_and_apply(genesis_account.signed_script_txn(
        encode_mint_lbr_to_address_script(&validator_account.address(), vec![], mint_amount),
        2,
    ));
    executor.new_block();

    let txn = register_validator_txn(
        &validator_account,
        vec![255; 32],
        vec![254; 32],
        vec![],
        vec![253; 32],
        vec![],
        0,
    );
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

    let txn = rotate_consensus_pubkey_txn(&validator_account, vec![251; 32], 1);
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
