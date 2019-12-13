// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::create_account_txn,
    executor::test_all_genesis,
};
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn create_account() {
    // create a FakeExecutor with a genesis from file
    test_all_genesis(|mut executor| {
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
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        let updated_receiver = executor
            .read_account_resource(&new_account)
            .expect("receiver must exist");
        assert_eq!(initial_amount, updated_receiver.balance(),);
        assert_eq!(sender_balance, updated_sender.balance(),);
        assert_eq!(11, updated_sender.sequence_number());
    });
}

#[test]
fn create_account_zero_balance() {
    // create a FakeExecutor with a genesis from file
    test_all_genesis(|mut executor| {
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
        println!("write set {:?}", output.write_set());
        executor.apply_write_set(output.write_set());

        // check that numbers in stored DB are correct
        let gas = output.gas_used();
        let sender_balance = 1_000_000 - initial_amount - gas;
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        let updated_receiver = executor
            .read_account_resource(&new_account)
            .expect("receiver must exist");
        assert_eq!(initial_amount, updated_receiver.balance());
        assert_eq!(sender_balance, updated_sender.balance());
        assert_eq!(11, updated_sender.sequence_number());
    });
}
