// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use language_common::error_codes::EINSUFFICIENT_BALANCE;
use move_ir::{assert_error_type, assert_no_error};

#[test]
fn cant_copy_resource() {
    let mut test_env = TestEnvironment::default();

    let program = b"
import 0x0.LibraAccount;
main() {
    let addr;
    let ten_coins;
    let i_created_money;
    addr = get_txn_sender();
    ten_coins = LibraAccount.withdraw_from_sender(10);
    i_created_money = copy(ten_coins);
    LibraAccount.deposit(copy(addr), move(ten_coins));
    LibraAccount.deposit(copy(addr), move(i_created_money));

    return;
}";

    assert_error_type!(
        test_env.run(to_script(program, vec![])),
        ErrorKind::InvalidCopy(_, _)
    )
}

#[test]
fn cant_double_deposit() {
    let program = b"
import 0x0.LibraAccount;
main() {
    let addr;
    let ten_coins;
    addr = get_txn_sender();
    ten_coins = LibraAccount.withdraw_from_sender(10);
    LibraAccount.deposit(copy(addr), move(ten_coins));
    LibraAccount.deposit(copy(addr), move(ten_coins));

    return;
}";
    assert_error_type!(run(program), ErrorKind::UseAfterMove(_, _))
}

#[test]
fn cant_overdraft() {
    let program = b"
import 0x0.LibraAccount;
main() {
    let addr;
    let sender_balance;
    let all_coins;
    let sender_new_balance;
    let one_coin;

    addr = get_txn_sender();

    sender_balance = LibraAccount.balance(copy(addr));

    all_coins = LibraAccount.withdraw_from_sender(move(sender_balance));

    sender_new_balance = LibraAccount.balance(copy(addr));
    assert(move(sender_new_balance) == 0, 41);

    one_coin = LibraAccount.withdraw_from_sender(1);

    return;
}";
    assert_error_type!(
        run(program),
        ErrorKind::AssertError(EINSUFFICIENT_BALANCE, _)
    )
}

#[test]
fn zero_payment() {
    let program = b"
import 0x0.LibraAccount;
import 0x0.LibraCoin;
main() {
    let addr;
    let sender_old_balance;
    let zero_resource;
    let sender_new_balance;

    addr = get_txn_sender();

    sender_old_balance = LibraAccount.balance(copy(addr));
    zero_resource = LibraCoin.zero();
    LibraAccount.deposit(copy(addr), move(zero_resource));

    sender_new_balance = LibraAccount.balance(move(addr));
    assert(move(sender_new_balance) == move(sender_old_balance), 42);

    return;
}";
    assert_error_type!(run(program), ErrorKind::AssertError(7, _))
}
