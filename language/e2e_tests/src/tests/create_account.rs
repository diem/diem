// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData, AccountResource},
    common_transactions::create_account_txn,
    executor::FakeExecutor,
};
use types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus},
};

#[test]
fn create_account() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    // create and publish a sender with 1_000_000 coins
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    let new_account = Account::new();
    let initial_amount = 1_000;
    let txn = create_account_txn(sender.account(), &new_account, 10, initial_amount);

    // execute transaction
    let txns: Vec<SignedTransaction> = vec![txn];
    let output = executor.execute_block(txns);
    let txn_output = output.get(0).expect("must have a transaction output");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
    );
    println!("write set {:?}", txn_output.write_set());
    executor.apply_write_set(txn_output.write_set());

    // check that numbers in stored DB are correct
    let gas = txn_output.gas_used();
    let sender_balance = 1_000_000 - initial_amount - gas;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_receiver = executor
        .read_account_resource(&new_account)
        .expect("receiver must exist");
    assert_eq!(
        initial_amount,
        AccountResource::read_balance(&updated_receiver)
    );
    assert_eq!(
        sender_balance,
        AccountResource::read_balance(&updated_sender)
    );
    assert_eq!(11, AccountResource::read_sequence_number(&updated_sender));
}

#[test]
fn create_account_zero_balance() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    // create and publish a sender with 1_000_000 coins
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    let new_account = Account::new();

    // define the arguments to the create account transaction
    let initial_amount = 0;
    let txn = create_account_txn(sender.account(), &new_account, 10, initial_amount);

    // execute transaction
    let txns: Vec<SignedTransaction> = vec![txn];
    let output = executor.execute_block(txns);
    let txn_output = output.get(0).expect("must have a transaction output");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
    );
    println!("write set {:?}", txn_output.write_set());
    executor.apply_write_set(txn_output.write_set());

    // check that numbers in stored DB are correct
    let gas = txn_output.gas_used();
    let sender_balance = 1_000_000 - initial_amount - gas;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_receiver = executor
        .read_account_resource(&new_account)
        .expect("receiver must exist");
    assert_eq!(
        initial_amount,
        AccountResource::read_balance(&updated_receiver)
    );
    assert_eq!(
        sender_balance,
        AccountResource::read_balance(&updated_sender)
    );
    assert_eq!(11, AccountResource::read_sequence_number(&updated_sender));
}
