// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use move_ir::assert_no_error;
use std::borrow::Borrow;

#[test]
fn peer_to_peer_payment_script() {
    let mut test_env = TestEnvironment::default();
    let recipient = test_env.accounts.get_address(1);
    let program = move_ir::stdlib::transaction_scripts::peer_to_peer_transfer_transaction_bincode(
        &recipient, 10,
    );
    let result = test_env.run_program(program);
    let gas_cost = match &result {
        Ok(res) => res.gas_used(),
        _ => 0,
    };

    assert_no_error!(result);

    let assert_balance = format!(
        "
import 0x0.LibraAccount;
main() {{
    let sender_address;
    let recipient_address;
    let sender_initial_balance;
    let transaction_cost;
    let sender_balance;
    let recipient_balance;

    sender_address = get_txn_sender();
    recipient_address = 0x{};
    sender_initial_balance = {};
    transaction_cost = {};
    sender_balance = LibraAccount.balance(move(sender_address));
    recipient_balance = LibraAccount.balance(move(recipient_address));

    assert(move(sender_balance) == copy(sender_initial_balance) - copy(transaction_cost) - 10, 77);
    assert(move(recipient_balance) == move(sender_initial_balance) + 10, 88);

    return;
}}",
        hex::encode(recipient),
        TestEnvironment::INITIAL_BALANCE,
        TestEnvironment::DEFAULT_GAS_COST * gas_cost,
    );

    assert_no_error!(test_env.run(to_script(assert_balance.as_bytes(), vec![])))
}

#[test]
fn peer_to_peer_payment_script_create_account() {
    let mut test_env = TestEnvironment::default();
    // some recipient whose account does not exist
    let recipient = test_env.accounts.fresh_account().addr;
    let program = move_ir::stdlib::transaction_scripts::peer_to_peer_transfer_transaction_bincode(
        &recipient, 10,
    );

    assert_no_error!(test_env.run_program(program));
}

#[test]
fn create_account_script() {
    let mut test_env = TestEnvironment::default();
    // some recipient whose account does not exist
    let fresh_account = test_env.accounts.fresh_account();
    let fresh_address = &fresh_account.addr;
    let program = move_ir::stdlib::transaction_scripts::create_account_transaction_bincode(
        fresh_address,
        TestEnvironment::DEFAULT_MAX_GAS + 100,
    );

    assert_no_error!(test_env.run_program(program));

    // make sure the account has been created by sending a transaction from it
    let sequence_number = 0;
    let txn = test_env.create_user_txn(
        to_script(b"main() { return; }", vec![]),
        fresh_address.clone(),
        fresh_account,
        sequence_number,
        TestEnvironment::DEFAULT_MAX_GAS,
        TestEnvironment::DEFAULT_GAS_COST,
    );

    assert_no_error!(test_env.run_txn(txn))
}

#[test]
fn rotate_authentication_key_script() {
    let mut test_env = TestEnvironment::default();
    let fresh_account = test_env.accounts.fresh_account();
    let new_authentication_key = fresh_account.pubkey.borrow().into();
    let program =
        move_ir::stdlib::transaction_scripts::rotate_authentication_key_transaction_bincode(
            new_authentication_key,
        );

    assert_no_error!(test_env.run_program(program));

    // we need to use the new key in order to send a transaction
    let old_account = test_env.accounts.get_account(0);
    let new_account = Account {
        addr: old_account.addr,
        privkey: fresh_account.privkey,
        pubkey: fresh_account.pubkey,
    };

    // make sure rotation worked by sending with the new key
    let sequence_number = 1;
    let txn = test_env.create_user_txn(
        to_standalone_script(b"main() { return; }"),
        old_account.addr,
        new_account,
        sequence_number,
        TestEnvironment::DEFAULT_MAX_GAS,
        TestEnvironment::DEFAULT_GAS_COST,
    );

    assert_no_error!(test_env.run_txn(txn))
}

// TODO: eliminate mint/this script
#[test]
fn mint_script() {
    let mut test_env = TestEnvironment::default();
    let recipient = test_env.accounts.get_address(1);
    let program = move_ir::stdlib::transaction_scripts::mint_transaction_bincode(&recipient, 10);
    let result = test_env.run_program(program);
    let gas_cost = match &result {
        Ok(res) => res.gas_used(),
        _ => 0,
    };

    assert_no_error!(result);

    let assert_balance = format!(
        "
import 0x0.LibraAccount;
main() {{
    let sender_address;
    let recipient_address;
    let sender_initial_balance;
    let transaction_cost;
    let sender_balance;
    let recipient_balance;

    sender_address = get_txn_sender();
    recipient_address = 0x{};
    sender_initial_balance = {};
    transaction_cost = {};
    sender_balance = LibraAccount.balance(move(sender_address));
    recipient_balance = LibraAccount.balance(move(recipient_address));

    assert(move(sender_balance) == copy(sender_initial_balance) - copy(transaction_cost), 77);
    assert(move(recipient_balance) == move(sender_initial_balance) + 10, 88);

    return;
}}",
        hex::encode(recipient),
        TestEnvironment::INITIAL_BALANCE,
        TestEnvironment::DEFAULT_GAS_COST * gas_cost,
    );

    assert_no_error!(test_env.run(to_script(assert_balance.as_bytes(), vec![])))
}
