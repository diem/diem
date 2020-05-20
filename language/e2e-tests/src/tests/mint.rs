// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account, AccountData},
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
    transaction_status_eq,
};
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};
use transaction_builder::*;

#[test]
fn mint_to_existing() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.
    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_association();

    // create and publish a sender with 1_000_000 coins
    let receiver = AccountData::new(1_000_000, 10);
    executor.add_account_data(&receiver);

    let mint_amount = 1_000;
    executor.execute_and_apply(association.signed_script_txn(
        encode_mint_lbr_to_address_script(&receiver.account().address(), vec![], mint_amount),
        1,
    ));

    // check that numbers in stored DB are correct
    let receiver_balance = 1_000_000 + mint_amount;

    let updated_sender = executor
        .read_account_resource(&association)
        .expect("sender balance must exist");
    let updated_receiver = executor
        .read_account_resource(receiver.account())
        .expect("receiver must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(receiver.account(), account::lbr_currency_code())
        .expect("receiver balance must exist");
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
    let association = Account::new_association();

    // create and publish a sender with TXN_RESERVED coins
    let new_account = Account::new();

    let mint_amount = TXN_RESERVED;
    executor.execute_and_apply(association.signed_script_txn(
        encode_mint_lbr_to_address_script(
            &new_account.address(),
            new_account.auth_key_prefix(),
            mint_amount,
        ),
        1,
    ));

    // check that numbers in stored DB are correct
    let receiver_balance = mint_amount;

    let updated_sender = executor
        .read_account_resource(&association)
        .expect("sender must exist");
    let updated_receiver = executor
        .read_account_resource(&new_account)
        .expect("receiver must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account, account::lbr_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(receiver_balance, updated_receiver_balance.coin());
    assert_eq!(2, updated_sender.sequence_number());
    assert_eq!(0, updated_receiver.sequence_number());

    // Mint can only be called from genesis address;
    let txn = new_account.signed_script_txn(
        encode_mint_lbr_to_address_script(&new_account.address(), vec![], mint_amount),
        0,
    );
    let output = executor.execute_transaction(txn);

    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::MISSING_DATA))
    ));
}
