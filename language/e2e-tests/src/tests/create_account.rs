// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::create_account_txn,
    executor::FakeExecutor,
};
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn create_account() {
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish a sender with 1_000_000 coins
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    let new_account = Account::new();
    let initial_amount = 1_000;
    let txn = create_account_txn(sender.account(), &new_account, 10, initial_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    println!("write set {:?}", output.write_set());
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let gas = output.gas_used();
    let sender_balance = 1_000_000 - initial_amount - gas;
    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(sender.account())
        .expect("sender must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account)
        .expect("receiver balance must exist");
    assert_eq!(initial_amount, updated_receiver_balance.coin(),);
    assert_eq!(sender_balance, updated_sender_balance.coin(),);
    assert_eq!(11, updated_sender.sequence_number());
}

#[test]
fn create_account_zero_balance() {
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish a sender with 1_000_000 coins
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    let new_account = Account::new();

    // define the arguments to the create account transaction
    let initial_amount = 0;
    let txn = create_account_txn(sender.account(), &new_account, 10, initial_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let gas = output.gas_used();
    let sender_balance = 1_000_000 - initial_amount - gas;
    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(sender.account())
        .expect("sender must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account)
        .expect("receiver balance must exist");
    assert_eq!(initial_amount, updated_receiver_balance.coin());
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(11, updated_sender.sequence_number());
}
