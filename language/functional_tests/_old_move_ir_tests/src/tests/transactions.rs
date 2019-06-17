// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use move_ir::{assert_error_type, assert_no_error};
use proptest::prelude::*;
use types::{
    account_address::AccountAddress,
    transaction::{TransactionArgument, TransactionPayload},
};

// no arguments on a void main() is good
#[test]
fn tx_no_args_good() {
    let mut test_env = TestEnvironment::default();

    assert_no_error!(test_env.run_with_arguments(vec![], to_script(b"main() { return; }", vec![])));
}

// one arg to a void main() is an error (integer arg)
#[test]
fn tx_no_args_bad1() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![TransactionArgument::U64(10)],
            to_script(b"main() { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    );
}

// no arg to a void main(u64) is an error
#[test]
fn tx_one_arg_bad1() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(vec![], to_script(b"main(value: u64) { return; }", vec![])),
        ErrorKind::BadTransactionArgs
    )
}

// two arg to a void main(u64) is an error
#[test]
fn tx_one_arg_bad2() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::U64(10),
                TransactionArgument::U64(1),
            ],
            to_script(b"main(value: u64) { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    );
}

// address arg to a void main(u64) is an error
#[test]
fn tx_one_arg_bad3() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![TransactionArgument::Address(AccountAddress::default())],
            to_script(b"main(value: u64) { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    );
}

// integer arg to main(u64) is good with assert true (value passed is expected)
#[test]
fn tx_one_arg_good1() {
    let mut test_env = TestEnvironment::default();

    assert_no_error!(test_env.run_with_arguments(
        vec![TransactionArgument::U64(10),],
        to_script(
            b"main(value: u64) { assert(copy(value) == 10, 42); return; }",
            vec![]
        )
    ))
}

// (u64, u64) args to main(u64, address) is an error
#[test]
fn tx_two_args_bad2() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::U64(10),
                TransactionArgument::U64(1),
            ],
            to_script(b"main(value: u64, addr: address) { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    )
}

// address arg to main(u64, address) is an error
#[test]
fn tx_two_args_bad3() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![TransactionArgument::Address(AccountAddress::default())],
            to_script(b"main(value: u64, addr: address) { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    )
}

// (address, u64) args to main(u64, address) is an error
#[test]
fn tx_two_args_bad4() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::Address(AccountAddress::default()),
                TransactionArgument::U64(10),
            ],
            to_script(b"main(value: u64, addr: address) { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    )
}

// (u64, address, u64) args to main(u64, address) is an error
#[test]
fn tx_two_args_bad5() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::U64(10),
                TransactionArgument::Address(AccountAddress::default()),
                TransactionArgument::U64(10),
            ],
            to_script(b"main(value: u64, addr: address) { return; }", vec![])
        ),
        ErrorKind::BadTransactionArgs
    )
}

// (u64, address) args to main(u64, address) is good - with assert not firing
#[test]
fn tx_two_args_good1() {
    let mut test_env = TestEnvironment::default();
    let account = test_env.accounts.get_account(1);
    let address = account.addr;
    let address_str = hex::encode(address);

    let program = format!(
        "
main(value: u64, addr: address) {{
    assert(copy(value) == 10, 42);
    assert(copy(addr) == 0x{}, 42);
    return;
}}
        ",
        address_str
    );

    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::U64(10),
            TransactionArgument::Address(address),
        ],
        to_script(program.as_bytes(), vec![])
    ));
}

// (u64, address) args to main(u64, address) is good - with assert firing on u64
#[test]
fn tx_two_args_good2() {
    let mut test_env = TestEnvironment::default();

    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::U64(1),
                TransactionArgument::Address(AccountAddress::default()),
            ],
            to_script(
                b"main(value: u64, addr: address) { assert(copy(value) == 10, 42); return; }",
                vec![],
            )
        ),
        ErrorKind::AssertError(_, _)
    );
}

// (u64, u64, address) args to main(u64, u64, address) is good
#[test]
fn tx_three_args_good1() {
    let mut test_env = TestEnvironment::default();

    let address = test_env.accounts.get_address(1);
    let address_str = hex::encode(address);

    let program = format!(
        "
main(value1: u64, value2: u64, addr: address) {{
    assert(copy(value1) + copy(value2) == 8, 42);
    assert(copy(addr) == 0x{}, 42);
    return;
}}
        ",
        address_str
    );

    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::U64(3),
            TransactionArgument::U64(5),
            TransactionArgument::Address(address),
        ],
        to_script(program.as_bytes(), vec![])
    ))
}

#[test]
fn write_set_txn_roundtrip() {
    // Creating a new test environment is expensive so do it outside the proptest environment.
    let test_env = TestEnvironment::default();

    proptest!(|(signed_txn in SignedTransaction::genesis_strategy())| {
        let write_set = match signed_txn.payload() {
            TransactionPayload::WriteSet(write_set) => write_set.clone(),
            TransactionPayload::Program(_) => unreachable!(
                "write set strategy should only generate write set transactions",
            ),
        };
        let output = test_env.eval_txn(signed_txn)
            .expect("write set transactions should succeed");
        prop_assert_eq!(output.write_set(), &write_set);
    });
}
