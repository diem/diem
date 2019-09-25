// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::mint_txn,
    executor::test_all_genesis,
    gas_costs::TXN_RESERVED,
    transaction_status_eq,
};
use types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn mint_to_existing() {
    // create a FakeExecutor with a genesis from file
    test_all_genesis(|mut executor| {
        let genesis_account = Account::new_association();

        // create and publish a sender with 1_000_000 coins
        let receiver = AccountData::new(1_000_000, 10);
        executor.add_account_data(&receiver);

        let mint_amount = 1_000;
        let txn = mint_txn(&genesis_account, receiver.account(), 1, mint_amount);

        // execute transaction
        let txns: Vec<SignedTransaction> = vec![txn];
        let output = executor.execute_block(txns);
        let txn_output = output.get(0).expect("must have a transaction output");
        assert_eq!(
            output[0].status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
        );
        println!("write set {:?}", txn_output.write_set());
        executor.apply_write_set(txn_output.write_set());

        // check that numbers in stored DB are correct
        let gas = txn_output.gas_used();
        let sender_balance = 1_000_000_000 - gas;
        let receiver_balance = 1_000_000 + mint_amount;

        let updated_sender = executor
            .read_account_resource(&genesis_account)
            .expect("sender must exist");
        let updated_receiver = executor
            .read_account_resource(receiver.account())
            .expect("receiver must exist");
        assert_eq!(sender_balance, updated_sender.balance());
        assert_eq!(receiver_balance, updated_receiver.balance());
        assert_eq!(2, updated_sender.sequence_number());
        assert_eq!(10, updated_receiver.sequence_number());
    });
}

#[test]
fn mint_to_new_account() {
    // create a FakeExecutor with a genesis from file
    test_all_genesis(|mut executor| {
        let genesis_account = Account::new_association();

        // create and publish a sender with TXN_RESERVED coins
        let new_account = Account::new();

        let mint_amount = TXN_RESERVED;
        let txn = mint_txn(&genesis_account, &new_account, 1, mint_amount);

        // execute transaction
        let txns: Vec<SignedTransaction> = vec![txn];
        let output = executor.execute_block(txns);
        let txn_output = output.get(0).expect("must have a transaction output");
        assert!(transaction_status_eq(
            &output[0].status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
        ));
        executor.apply_write_set(txn_output.write_set());

        // check that numbers in stored DB are correct
        let gas = txn_output.gas_used();
        let sender_balance = 1_000_000_000 - gas;
        let receiver_balance = mint_amount;

        let updated_sender = executor
            .read_account_resource(&genesis_account)
            .expect("sender must exist");
        let updated_receiver = executor
            .read_account_resource(&new_account)
            .expect("receiver must exist");
        assert_eq!(sender_balance, updated_sender.balance());
        assert_eq!(receiver_balance, updated_receiver.balance());
        assert_eq!(2, updated_sender.sequence_number());
        assert_eq!(0, updated_receiver.sequence_number());

        // Mint can only be called from genesis address;
        let txn = mint_txn(&new_account, &new_account, 0, mint_amount);
        let txns: Vec<SignedTransaction> = vec![txn];
        let output = executor.execute_block(txns);

        assert!(transaction_status_eq(
            &output[0].status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::MISSING_DATA))
        ));
    });
}
