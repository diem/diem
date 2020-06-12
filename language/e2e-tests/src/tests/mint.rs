// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account, AccountData},
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
    transaction_status_eq,
};
use libra_types::{
    account_config,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};
use transaction_builder::*;

#[test]
fn tiered_mint_designated_dealer() {
    let mut executor = FakeExecutor::from_genesis_file();
    let blessed = Account::new_blessed_tc();
    // account to represent designated dealer
    let dd = Account::new();
    executor.execute_and_apply(blessed.signed_script_txn(
        encode_create_designated_dealer(
            account_config::coin1_tag(),
            0,
            *dd.address(),
            dd.auth_key_prefix(),
        ),
        0,
    ));
    let mint_amount = 1_000;
    let tier_index = 0;
    executor.execute_and_apply(blessed.signed_script_txn(
        encode_tiered_mint(
            account_config::coin1_tag(),
            1,
            *dd.address(),
            mint_amount,
            tier_index,
        ),
        1,
    ));
    let dd_post_mint = executor
        .read_account_resource(&dd)
        .expect("receiver must exist");
    let dd_balance = executor
        .read_balance_resource(&dd, account::coin1_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(mint_amount, dd_balance.coin());
    assert_eq!(0, dd_post_mint.sequence_number());
}

#[test]
fn mint_to_existing() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.
    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_blessed_tc();

    // create and publish a sender with 1_000_000 coins
    let receiver = AccountData::new(1_000_000, 10);
    executor.add_account_data(&receiver);

    let mint_amount = 1_000;
    executor.execute_and_apply(association.signed_script_txn(
        encode_mint_script(
            account_config::lbr_type_tag(),
            &receiver.account().address(),
            vec![],
            mint_amount,
        ),
        0,
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
    assert_eq!(1, updated_sender.sequence_number());
    assert_eq!(10, updated_receiver.sequence_number());
}

#[test]
fn mint_to_new_account() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.

    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_blessed_tc();

    // create and publish a sender with TXN_RESERVED coins
    let new_account = Account::new();

    let mint_amount = TXN_RESERVED;
    executor.execute_and_apply(association.signed_script_txn(
        encode_mint_script(
            account_config::lbr_type_tag(),
            &new_account.address(),
            new_account.auth_key_prefix(),
            mint_amount,
        ),
        0,
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
    assert_eq!(1, updated_sender.sequence_number());
    assert_eq!(0, updated_receiver.sequence_number());

    // Mint can only be called from genesis address;
    let txn = new_account.signed_script_txn(
        encode_mint_script(
            account_config::lbr_type_tag(),
            &new_account.address(),
            vec![],
            mint_amount,
        ),
        0,
    );
    let output = executor.execute_transaction(txn);

    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::MISSING_DATA))
    ));
}

#[test]
fn tiered_update_exchange_rate() {
    let mut executor = FakeExecutor::from_genesis_file();
    let blessed = Account::new_blessed_tc();

    // set coin1 rate to 1.23 LBR
    executor.execute_and_apply(blessed.signed_script_txn(
        encode_update_exchange_rate(account_config::coin1_tag(), 0, 123, 100),
        0,
    ));
    let post_update = executor
        .read_account_resource(&blessed)
        .expect("blessed executed txn");
    assert_eq!(1, post_update.sequence_number());
}
