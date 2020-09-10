// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::{
    account::{self, Account},
    common_transactions::create_account_txn,
    executor::FakeExecutor,
};
use libra_types::{account_config, transaction::TransactionStatus, vm_status::KeptVMStatus};

#[test]
fn create_account() {
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish a sender with 1_000_000 coins
    let sender = Account::new_blessed_tc();
    let new_account = Account::new();

    // define the arguments to the create account transaction
    let initial_amount = 0;
    let txn = create_account_txn(
        &sender,
        &new_account,
        0,
        initial_amount,
        account_config::lbr_type_tag(),
    );

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let updated_sender = executor
        .read_account_resource(&sender)
        .expect("sender must exist");

    let updated_receiver_balance = executor
        .read_balance_resource(&new_account, account::lbr_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(initial_amount, updated_receiver_balance.coin());
    assert_eq!(1, updated_sender.sequence_number());
}
