// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account, AccountData},
    common_transactions::create_account_txn,
    executor::FakeExecutor,
};
use libra_types::{
    account_config::lbr_type_tag,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};
use move_vm_types::values::Value;

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
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let sender_balance = 1_000_000 - initial_amount;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(sender.account(), account::lbr_currency_code())
        .expect("sender balance must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account, account::lbr_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(initial_amount, updated_receiver_balance.coin(),);
    assert_eq!(sender_balance, updated_sender_balance.coin(),);
    assert_eq!(11, updated_sender.sequence_number());
}

#[test]
fn create_account_with_exec() {
    let mut executor = FakeExecutor::from_genesis_file();
    let new_account = Account::new();
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // create an hosted account via function invokation.
    executor.exec(
        "LibraAccount",
        "create_unhosted_account",
        vec![lbr_type_tag()],
        vec![
            Value::transaction_argument_signer_reference(*sender.address()),
            Value::address(*new_account.address()),
            Value::vector_u8(new_account.auth_key_prefix()),
            Value::bool(false),
        ],
        new_account.address(),
    );

    // check that numbers in stored DB are correct
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account, account::lbr_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(0, updated_receiver_balance.coin());
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
    let sender_balance = 1_000_000 - initial_amount;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(sender.account(), account::lbr_currency_code())
        .expect("sender balance must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account, account::lbr_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(initial_amount, updated_receiver_balance.coin());
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(11, updated_sender.sequence_number());
}
