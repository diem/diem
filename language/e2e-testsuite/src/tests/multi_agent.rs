// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for multi-agent transactions.

use diem_transaction_builder::stdlib::*;
use diem_types::{
    account_config, test_helpers::transaction_test_helpers, transaction::TransactionStatus,
    vm_status::KeptVMStatus,
};
use language_e2e_tests::{
    account::{self, xdx_currency_code, xus_currency_code, Account},
    common_transactions::{
        multi_agent_mint_txn, multi_agent_p2p_txn, multi_agent_swap_script, multi_agent_swap_txn,
    },
    current_function_name,
    executor::FakeExecutor,
};

#[test]
fn multi_agent_mint() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let tc = Account::new_blessed_tc();

    // account to represent designated dealer
    let dd = executor.create_raw_account();
    executor.execute_and_apply(
        tc.transaction()
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

    // account to represent VASP
    let vasp = executor.create_raw_account();
    executor.execute_and_apply(
        tc.transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *vasp.address(),
                vasp.auth_key_prefix(),
                vec![],
                false, // add_all_currencies
            ))
            .sequence_number(1)
            .sign(),
    );

    let mint_amount = 1_000;
    let tier_index = 0;
    let txn = multi_agent_mint_txn(&tc, &dd, &vasp, 2, mint_amount, tier_index);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());

    let updated_tc = executor.read_account_resource(&tc).expect("tc must exist");
    let updated_dd = executor.read_account_resource(&dd).expect("dd must exist");
    let updated_vasp = executor
        .read_account_resource(&vasp)
        .expect("vasp must exist");
    let updated_dd_balance = executor
        .read_balance_resource(&dd, account::xus_currency_code())
        .expect("dd balance must exist");
    let updated_vasp_balance = executor
        .read_balance_resource(&vasp, account::xus_currency_code())
        .expect("vasp balance must exist");
    assert_eq!(0, updated_dd_balance.coin());
    assert_eq!(mint_amount, updated_vasp_balance.coin());
    assert_eq!(3, updated_tc.sequence_number());
    assert_eq!(0, updated_dd.sequence_number());
    assert_eq!(0, updated_vasp.sequence_number());
    assert_eq!(1, updated_dd.sent_events().count());
    assert_eq!(1, updated_vasp.received_events().count());
}

#[test]
fn multi_agent_swap() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    // create and publish a sender with 1_000_010 XUS coins
    // and a secondary signer with 100_100 XDX coins.
    let mut sender = executor.create_raw_account_data(1_000_010, 10);
    let mut secondary_signer = executor.create_xdx_raw_account_data(100_100, 100);
    sender.add_balance_currency(xdx_currency_code());
    secondary_signer.add_balance_currency(xus_currency_code());

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);

    let xus_amount = 10;
    let xdx_amount = 100;

    let txn = multi_agent_swap_txn(
        sender.account(),
        secondary_signer.account(),
        10,
        xus_amount,
        xdx_amount,
    );
    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let sender_xus_balance = 1_000_010 - xus_amount;
    let secondary_signer_xdx_balance = 100_100 - xdx_amount;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_sender_xus_balance = executor
        .read_balance_resource(sender.account(), account::xus_currency_code())
        .expect("sender xus balance must exist");
    let updated_sender_xdx_balance = executor
        .read_balance_resource(sender.account(), account::xdx_currency_code())
        .expect("sender xdx balance must exist");
    let updated_secondary_signer = executor
        .read_account_resource(secondary_signer.account())
        .expect("secondary signer must exist");
    let updated_secondary_signer_xus_balance = executor
        .read_balance_resource(secondary_signer.account(), account::xus_currency_code())
        .expect("secondary signer xus balance must exist");
    let updated_secondary_signer_xdx_balance = executor
        .read_balance_resource(secondary_signer.account(), account::xdx_currency_code())
        .expect("secondary signer xdx balance must exist");
    assert_eq!(sender_xus_balance, updated_sender_xus_balance.coin());
    assert_eq!(xdx_amount, updated_sender_xdx_balance.coin());
    assert_eq!(xus_amount, updated_secondary_signer_xus_balance.coin());
    assert_eq!(
        secondary_signer_xdx_balance,
        updated_secondary_signer_xdx_balance.coin()
    );
    assert_eq!(11, updated_sender.sequence_number());
    assert_eq!(100, updated_secondary_signer.sequence_number());
    assert_eq!(1, updated_sender.received_events().count(),);
    assert_eq!(1, updated_sender.sent_events().count());
    assert_eq!(1, updated_secondary_signer.received_events().count());
    assert_eq!(1, updated_secondary_signer.sent_events().count());
}

#[test]
fn multi_agent_p2p() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    // create and publish a sender with 1_000_010 XUS coins
    // and a secondary signer with 10 XUS coins.
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(10, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);

    let amount = 10;

    let txn = multi_agent_p2p_txn(sender.account(), secondary_signer.account(), 10, amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let sender_balance = 1_000_010 - amount;
    let secondary_signer_balance = 10 + amount;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(sender.account(), account::xus_currency_code())
        .expect("sender xus balance must exist");
    let updated_secondary_signer = executor
        .read_account_resource(secondary_signer.account())
        .expect("secondary signer must exist");
    let updated_secondary_signer_balance = executor
        .read_balance_resource(secondary_signer.account(), account::xus_currency_code())
        .expect("secondary signer xus balance must exist");

    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(
        secondary_signer_balance,
        updated_secondary_signer_balance.coin()
    );
    assert_eq!(11, updated_sender.sequence_number());
    assert_eq!(100, updated_secondary_signer.sequence_number());
    assert_eq!(0, updated_sender.received_events().count(),);
    assert_eq!(1, updated_sender.sent_events().count());
    assert_eq!(1, updated_secondary_signer.received_events().count());
    assert_eq!(0, updated_secondary_signer.sent_events().count());
}

#[test]
fn multi_agent_wrong_number_of_signers() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);
    let third_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);
    executor.add_account_data(&third_signer);

    // Number of secondary signers given is 2 but the script only allows one secondary signer.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*secondary_signer.address(), *third_signer.address()],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![
            &secondary_signer.account().privkey,
            &third_signer.account().privkey,
        ],
        vec![
            secondary_signer.account().pubkey.clone(),
            third_signer.account().pubkey.clone(),
        ],
        Some(multi_agent_swap_script(10, 0)), // swap between two accounts
    );
    let output = executor.execute_transaction(signed_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::MiscellaneousError)
    );
}
