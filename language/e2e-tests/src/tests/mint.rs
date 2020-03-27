// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::mint_txn,
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
    transaction_status_eq,
};
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};
use vm_genesis::ASSOCIATION_INIT_BALANCE;

#[test]
fn mint_to_existing() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();

    // create and publish a sender with 1_000_000 coins
    let receiver = AccountData::new(1_000_000, 10);
    executor.add_account_data(&receiver);

    let mint_amount = 1_000;
    let txn = mint_txn(&genesis_account, receiver.account(), 1, mint_amount);

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
    let sender_balance = ASSOCIATION_INIT_BALANCE - gas;
    let receiver_balance = 1_000_000 + mint_amount;

    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(&genesis_account)
        .expect("sender balance must exist");
    let (updated_receiver, updated_receiver_balance) = executor
        .read_account_info(receiver.account())
        .expect("receiver must exist");
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(receiver_balance, updated_receiver_balance.coin());
    assert_eq!(2, updated_sender.sequence_number());
    assert_eq!(10, updated_receiver.sequence_number());
}

#[test]
fn mint_to_new_account() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.

    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();

    // create and publish a sender with TXN_RESERVED coins
    let new_account = Account::new();

    let mint_amount = TXN_RESERVED;
    let txn = mint_txn(&genesis_account, &new_account, 1, mint_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    ));
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let gas = output.gas_used();
    let sender_balance = ASSOCIATION_INIT_BALANCE - gas;
    let receiver_balance = mint_amount;

    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(&genesis_account)
        .expect("sender must exist");
    let (updated_receiver, updated_receiver_balance) = executor
        .read_account_info(&new_account)
        .expect("receiver must exist");
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(receiver_balance, updated_receiver_balance.coin());
    assert_eq!(2, updated_sender.sequence_number());
    assert_eq!(0, updated_receiver.sequence_number());

    // Mint can only be called from genesis address;
    let txn = mint_txn(&new_account, &new_account, 0, mint_amount);
    let output = executor.execute_transaction(txn);

    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::MISSING_DATA))
    ));
}
