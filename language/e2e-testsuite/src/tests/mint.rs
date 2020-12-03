// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    account_config,
    transaction::TransactionStatus,
    vm_status::{known_locations, KeptVMStatus},
};
use language_e2e_tests::{
    account::{self, Account},
    current_function_name,
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
    transaction_status_eq,
};
use transaction_builder::*;

#[test]
fn tiered_mint_designated_dealer() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();

    // account to represent designated dealer
    let dd = executor.create_raw_account();
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_designated_dealer_script(
                account_config::xus_tag(),
                0,
                *dd.address(),
                dd.auth_key_prefix(),
                vec![],
                false, // add_all_currencies
            ))
            .sequence_number(0)
            .sign(),
    );
    let mint_amount_one = 1_000;
    let tier_index = 0;
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_tiered_mint_script(
                account_config::xus_tag(),
                1,
                *dd.address(),
                mint_amount_one,
                tier_index,
            ))
            .sequence_number(1)
            .sign(),
    );
    let dd_post_mint = executor
        .read_account_resource(&dd)
        .expect("receiver must exist");
    let dd_balance = executor
        .read_balance_resource(&dd, account::xus_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(mint_amount_one, dd_balance.coin());
    assert_eq!(0, dd_post_mint.sequence_number());

    // --------------
    let mint_amount_two = 5_000_000;
    let tier_index = 3;
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_tiered_mint_script(
                account_config::xus_tag(),
                2,
                *dd.address(),
                mint_amount_two,
                tier_index,
            ))
            .sequence_number(2)
            .sign(),
    );
    let dd_balance = executor
        .read_balance_resource(&dd, account::xus_currency_code())
        .expect("receiver balance must exist");
    assert_eq!(mint_amount_one + mint_amount_two, dd_balance.coin());

    // -------------- invalid tier index
    let tier_index = 4;
    let output = &executor.execute_transaction(
        blessed
            .transaction()
            .script(encode_tiered_mint_script(
                account_config::xus_tag(),
                3,
                *dd.address(),
                mint_amount_one,
                tier_index,
            ))
            .sequence_number(3)
            .sign(),
    );
    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(KeptVMStatus::MoveAbort(
            known_locations::designated_dealer_module_abort(),
            775
        )),
    ));
}

#[test]
fn mint_to_existing_not_dd() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();

    // create and publish a sender with 1_000_000 coins
    let receiver = executor.create_raw_account();

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *receiver.address(),
                receiver.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(0)
            .sign(),
    );

    let mint_amount = 1_000;
    let output = executor.execute_transaction(
        blessed
            .transaction()
            .script(encode_tiered_mint_script(
                account_config::xus_tag(),
                0,
                *receiver.address(),
                mint_amount,
                4,
            ))
            .sequence_number(1)
            .sign(),
    );
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::MoveAbort(
            known_locations::designated_dealer_module_abort(),
            5
        )),
    );
}

#[test]
fn mint_to_new_account() {
    // create a FakeExecutor with a genesis from file
    // We can't run mint test on terraform genesis as we don't have the private key to sign the
    // mint transaction.

    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let tc = Account::new_blessed_tc();

    // create and publish a sender with TXN_RESERVED coins
    let new_account = executor.create_raw_account();

    let mint_amount = TXN_RESERVED;
    let output = executor.execute_transaction(
        tc.transaction()
            .script(encode_tiered_mint_script(
                account_config::xus_tag(),
                0,
                *new_account.address(),
                mint_amount,
                4,
            ))
            .sequence_number(0)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::MoveAbort(
            known_locations::designated_dealer_module_abort(),
            5
        )),
    );
}

#[test]
fn tiered_update_exchange_rate() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();

    // set xus rate to 1.23 XUS
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_update_exchange_rate_script(
                account_config::xus_tag(),
                0,
                123,
                100,
            ))
            .sequence_number(0)
            .sign(),
    );
    let post_update = executor
        .read_account_resource(&blessed)
        .expect("blessed executed txn");
    assert_eq!(1, post_update.sequence_number());
}
