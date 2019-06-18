// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use language_common::error_codes::*;
use move_ir::{assert_error_type, assert_no_error};
use types::transaction::TransactionArgument;

#[test]
fn increment_sequence_number_on_tx_script_success() {
    let mut test_env = TestEnvironment::default();

    let sequence_number = 0;
    assert_no_error!(test_env
        .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![]),));

    // now we need to run with sequence number 1
    let sequence_number = 1;
    assert_no_error!(test_env.run_with_sequence_number(
        sequence_number,
        to_script(
            b"
import 0x0.LibraAccount;
main() {
   let transaction_sequence_number;
   let sender;
   let sequence_number;

   transaction_sequence_number = get_txn_sequence_number();
   assert(copy(transaction_sequence_number) == 1, 42);

   sender = get_txn_sender();
   sequence_number = LibraAccount.sequence_number(move(sender));
   assert(move(sequence_number) == 1, 43);

  return;
}",
            vec![]
        ),
    ))
}

#[test]
fn increment_sequence_number_on_tx_script_failure() {
    let mut test_env = TestEnvironment::default();

    let sequence_number = 0;
    assert_error_type!(
        test_env.run_with_sequence_number(
            sequence_number,
            to_script(
                b"
main() {
  assert(false, 77);
  return;
}
",
                vec![]
            )
        ),
        ErrorKind::AssertError(77, _)
    );

    // there was a failure during the transaction script, but
    // sequence number should still be bumped
    let sequence_number = 1;
    assert_no_error!(test_env
        .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![]),))
}

#[test]
fn charge_more_gas_on_tx_script_failure1() {
    // Make sure that we charge more for a transaction that was aborted
    let mut test_env = TestEnvironment::default();

    let program = b"
    main(abrt: u64) {
      assert(move(abrt) == 0, 78);
      return;
    }";
    let result = test_env.run_with_arguments(
        vec![TransactionArgument::U64(0)],
        to_script(program, vec![]),
    );
    let good_gas_cost = match &result {
        Ok(res) => res.gas_used(),
        _ => 0,
    };
    assert_no_error!(result);

    // Fail this one
    assert_error_type!(
        test_env.run_with_arguments(
            vec![TransactionArgument::U64(1)],
            to_script(program, vec![])
        ),
        ErrorKind::AssertError(78, _)
    );

    let verify_script = b"
    import 0x0.LibraAccount;
    main(initial_balance: u64, good_gas_fees: u64) {
      let sender;
      let sender_balance;
      sender = get_txn_sender();
      sender_balance = LibraAccount.balance(copy(sender));
      assert(move(initial_balance) - move(good_gas_fees) < move(sender_balance), 79);
      return;
    }";
    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(2 * good_gas_cost * TestEnvironment::DEFAULT_GAS_COST)
        ],
        to_script(verify_script, vec![])
    ));
}

#[test]
fn charge_more_gas_on_tx_script_failure2() {
    // This then verifies that the verification would have failed if both transactions
    // had succeeded in `charge_more_gas_on_tx_script_failure1`.
    let mut test_env = TestEnvironment::default();

    let program = b"
    main(abrt: u64) {
      assert(move(abrt) == 0, 80);
      return;
    }";
    let result = test_env.run_with_arguments(
        vec![TransactionArgument::U64(0)],
        to_script(program, vec![]),
    );
    let good_gas_cost = match &result {
        Ok(res) => res.gas_used(),
        _ => 0,
    };
    assert_no_error!(result);

    assert_no_error!(test_env.run_with_arguments(
        vec![TransactionArgument::U64(0)],
        to_script(program, vec![])
    ));

    let verify_script = b"
    import 0x0.LibraAccount;
    main(initial_balance: u64, good_gas_fees: u64) {
      let sender;
      let sender_balance;
      sender = get_txn_sender();
      sender_balance = LibraAccount.balance(copy(sender));
      assert(move(initial_balance) - move(good_gas_fees) < move(sender_balance), 81);
      return;
    }";
    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
                TransactionArgument::U64(2 * good_gas_cost * TestEnvironment::DEFAULT_GAS_COST)
            ],
            to_script(verify_script, vec![])
        ),
        ErrorKind::AssertError(81, _)
    );
}

#[test]
fn dont_increment_sequence_number_on_sequence_number_too_new() {
    let mut test_env = TestEnvironment::default();

    let sequence_number = 7;
    assert_error_type!(
        test_env.run_with_sequence_number(
            sequence_number,
            to_script(
                b"
main() {
  assert(false, 77);
  return;
}
",
                vec![]
            )
        ),
        ErrorKind::AssertError(ESEQUENCE_NUMBER_TOO_NEW, _)
    );

    // running with 0 should succeed because sequence number wasn't bumped
    let sequence_number = 0;
    assert_no_error!(test_env
        .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![]),))
}

#[test]
fn dont_increment_sequence_number_on_sequence_number_too_old() {
    let mut test_env = TestEnvironment::default();

    let sequence_number = 0;
    assert_no_error!(test_env
        .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![]),));

    // running with 0 should fail
    let sequence_number = 0;
    assert_error_type!(
        test_env
            .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![])),
        ErrorKind::AssertError(ESEQUENCE_NUMBER_TOO_OLD, _)
    );

    // but running with 1 should succeed
    let sequence_number = 1;
    assert_no_error!(test_env
        .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![]),))
}

// You can call your own epilogue if you want to, but your sequence number will only be bumped once
#[test]
fn calling_own_epilogue_bumps_sequence_number_once() {
    let mut test_env = TestEnvironment::default();

    let sequence_number = 0;
    assert_no_error!(test_env.run_with_sequence_number(
        sequence_number,
        to_script(
            b"
import 0x0.LibraAccount;
main() {
  LibraAccount.epilogue();
  LibraAccount.epilogue();
  LibraAccount.epilogue();

  return;
}",
            vec![]
        ),
    ));

    let sequence_number = 1;
    assert_no_error!(test_env
        .run_with_sequence_number(sequence_number, to_script(b"main() { return; }", vec![]),))
}

#[test]
fn gas_charge_different() {
    let mut test_env1 = TestEnvironment::default();
    let mut test_env2 = TestEnvironment::default();
    let max_gas_deposit_fee = TestEnvironment::DEFAULT_MAX_GAS * TestEnvironment::DEFAULT_GAS_COST;
    let program1 = b"main() { return; }";
    let program2 = b"main() { let x; x = 32 + 10; return; }";
    let verifier_program = b"
        import 0x0.LibraAccount;
        main(original_balance: u64, max_gas_fee: u64, actual_gas_fee: u64) {
            let sender;
            let sender_balance;
            sender = get_txn_sender();
            sender_balance = LibraAccount.balance(copy(sender));
            assert(copy(original_balance) - move(max_gas_fee) < copy(sender_balance), 66);
            assert(move(original_balance) - move(actual_gas_fee) == move(sender_balance), 66);

            return;
        }";

    // Run the first program
    let result1 = test_env1.run(to_script(program1, vec![])).unwrap();
    let gas_used1 = result1.gas_used();
    let gas_used_fee1 = gas_used1 * TestEnvironment::DEFAULT_GAS_COST;
    // Verify that this gas amount is correct
    assert_no_error!(test_env1.run_with_arguments(
        vec![
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(max_gas_deposit_fee),
            TransactionArgument::U64(gas_used_fee1),
        ],
        to_script(verifier_program, vec![]),
    ));

    // Now run the second program. Use a different test env so we don't have to do accounting for
    // the cost of the verifier program.
    let result2 = test_env2.run(to_script(program2, vec![])).unwrap();
    let gas_used2 = result2.gas_used();
    let gas_used_fee2 = gas_used2 * TestEnvironment::DEFAULT_GAS_COST;
    // Verify that this gas amount for the second when is correct as well
    assert_no_error!(test_env2.run_with_arguments(
        vec![
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(max_gas_deposit_fee),
            TransactionArgument::U64(gas_used_fee2),
        ],
        to_script(verifier_program, vec![]),
    ));
    // Make sure that the first one is less than the second. These numbers have been verified.
    assert!(gas_used1 < gas_used2);
}

#[test]
fn gas_charge_accurate() {
    let mut test_env = TestEnvironment::default();
    let program1 = b"main() { return; }";
    // Ensures that we are not just charging max_gas for the transaction.
    // Ensures that the account was deducted for the gas fee
    let verifier_program = b"
        import 0x0.LibraAccount;
        main(original_balance: u64, max_gas_fee: u64, actual_gas_fee: u64) {
            let sender;
            let sender_balance;
            sender = get_txn_sender();
            sender_balance = LibraAccount.balance(copy(sender));
            assert(copy(original_balance) - move(max_gas_fee) < copy(sender_balance), 66);
            assert(move(original_balance) - move(actual_gas_fee) == move(sender_balance), 66);

            return;
        }";
    let result1 = test_env.run(to_script(program1, vec![])).unwrap();
    let gas_used1 = result1.gas_used();
    let max_gas_deposit_fee = TestEnvironment::DEFAULT_MAX_GAS * TestEnvironment::DEFAULT_GAS_COST;
    let gas_used_fee = gas_used1 * TestEnvironment::DEFAULT_GAS_COST;
    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(max_gas_deposit_fee),
            TransactionArgument::U64(gas_used_fee),
        ],
        to_script(verifier_program, vec![]),
    ));
}

#[test]
fn gas_deposit_withdraws() {
    let mut test_env = TestEnvironment::default();

    let result = test_env
        .run(to_script(b"main() { return; }", vec![]))
        .unwrap();
    let gas_used = result.gas_used();

    // sender balance should be less the gas deposit after transaction execution
    let gas_deposit_fee = TestEnvironment::DEFAULT_GAS_COST * gas_used;
    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(gas_deposit_fee),
        ],
        to_script(
            b"
import 0x0.LibraAccount;
main(original_balance: u64, gas_deposit_amount: u64) {
  let sender;
  let sender_balance;

  sender = get_txn_sender();
  sender_balance = LibraAccount.balance(copy(sender));
  assert(move(sender_balance) == move(original_balance) - move(gas_deposit_amount), 66);

  return;
}",
            vec![]
        ),
    ))
}

#[test]
fn revert_tx_script_state_changes_after_failure() {
    let mut test_env = TestEnvironment::default();
    let recipient = test_env.accounts.get_address(1);

    let amount = 10;
    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::Address(recipient),
                TransactionArgument::U64(amount)
            ],
            to_script(
                b"
import 0x0.LibraAccount;
main(payee: address, amount: u64) {
  LibraAccount.pay_from_sender(move(payee), move(amount));
  assert(false, 66);
  return;
}
",
                vec![]
            )
        ),
        ErrorKind::AssertError(66, _)
    );

    // now we need to run with sequence number 1
    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::Address(recipient),
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE)
        ],
        to_script(
            b"
import 0x0.LibraAccount;
main(recipient: address, recipient_original_balance: u64) {
   let recipient_balance;
   recipient_balance = LibraAccount.balance(move(recipient));
   assert(move(recipient_balance) == move(recipient_original_balance), 55);

  return;
}",
            vec![]
        ),
    ))
}

#[test]
fn revert_tx_script_state_changes_after_failed_epilogue() {
    let mut test_env = TestEnvironment::default();
    let recipient = test_env.accounts.get_address(1);

    // transfer all of the sender's funds to recipients. this script will execute successfully, but
    // the epilogue will fail because the user spent his gas deposit. the VM should revert the state
    // changes and re-execute the epilogue
    let amount = TestEnvironment::INITIAL_BALANCE;
    assert_error_type!(
        test_env.run_with_arguments(
            vec![
                TransactionArgument::Address(recipient),
                TransactionArgument::U64(amount)
            ],
            to_script(
                b"
                import 0x0.LibraAccount;
                main(payee: address, amount: u64) {
                    LibraAccount.pay_from_sender(move(payee), move(amount));
                    return;
                }",
                vec![]
            )
        ),
        ErrorKind::OutOfGas
    );

    // We need to calculate the gas used in order to verify that the correct amount has been
    // debited below. Sadly the only way of doing this is re-running the transaction since the
    // previous run ran out of gas.
    let gas_used = {
        let mut test_env2 = TestEnvironment::default();
        let recipient = test_env2.accounts.get_address(1);
        let amount = 1;
        let result = test_env2
            .run_with_arguments(
                vec![
                    TransactionArgument::Address(recipient),
                    TransactionArgument::U64(amount),
                ],
                to_script(
                    b"
                import 0x0.LibraAccount;
                main(payee: address, amount: u64) {
                    LibraAccount.pay_from_sender(move(payee), move(amount));
                    return;
                }",
                    vec![],
                ),
            )
            .unwrap();
        result.gas_used()
    };

    let gas_deposit_fee = TestEnvironment::DEFAULT_GAS_COST * gas_used;
    // this script checks that the state changes from tx1's script were actually reverted; the
    // recipient's balance should be the same as the initial state, and the sender's balance should
    // be the same as the initial state less the gas deposit.
    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::Address(recipient),
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(gas_deposit_fee),
        ],
        to_script(
            b"
import 0x0.LibraAccount;
main(recipient: address, original_balance: u64, gas_deposit_amount: u64) {
  let recipient_balance;
  let sender;
  let sender_balance;
  let sender_sequence_number;

  recipient_balance = LibraAccount.balance(move(recipient));

  sender = get_txn_sender();
  sender_balance = LibraAccount.balance(copy(sender));
  assert(move(sender_balance) == move(original_balance) - move(gas_deposit_amount), 66);

  sender_sequence_number = LibraAccount.sequence_number(move(sender));
  assert(move(sender_sequence_number) == 1, 77);

  return;
}",
            vec![]
        ),
    ))
}

#[test]
fn recursion_out_of_gas_charges_max_gas() {
    let mut test_env = TestEnvironment::default();
    let sender = test_env.accounts.get_account(0).addr;

    let program = format!(
        "
modules:
module Looper {{
  public run_loop(n: u64) {{
    while (true) {{

    }}
    return;
  }}

}}

script:
import 0x{0}.Looper;
main() {{
    Looper.run_loop(5);
    return;
}}
",
        hex::encode(sender)
    );

    let gas_amount = 423;
    assert_error_type!(
        test_env.run_with_max_gas_amount(gas_amount, to_standalone_script(program.as_bytes())),
        ErrorKind::OutOfGas
    );

    let verify_script = b"
    import 0x0.LibraAccount;
    main(initial_balance: u64, gas_fees: u64) {
      let sender;
      let sender_balance;
      sender = get_txn_sender();
      sender_balance = LibraAccount.balance(copy(sender));
      assert(move(initial_balance) - move(gas_fees) == move(sender_balance), 101);
      return;
    }";
    let gas_fee = gas_amount * TestEnvironment::DEFAULT_GAS_COST;
    assert_no_error!(test_env.run_with_arguments(
        vec![
            TransactionArgument::U64(TestEnvironment::INITIAL_BALANCE),
            TransactionArgument::U64(gas_fee)
        ],
        to_script(verify_script, vec![])
    ));
}

// TODO don't increment sequence number after:
// bad signature
// bad auth key
// can't pay gas deposit

// TODO: do increment sequence number after:
// parse error
// module publish error
// bytecode verification error
// non-assert runtime error
