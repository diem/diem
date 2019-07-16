// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountData,
    assert_prologue_disparity, assert_prologue_parity,
    common_transactions::*,
    compile::{compile_program_with_address, compile_script},
    executor::FakeExecutor,
};
use assert_matches::assert_matches;
use bytecode_verifier::VerifiedModule;
use compiler::Compiler;
use config::config::{NodeConfigHelpers, VMPublishingOption};
use crypto::signing::KeyPair;
use std::collections::HashSet;
use tiny_keccak::Keccak;
use types::{
    account_address::AccountAddress,
    test_helpers::transaction_test_helpers,
    transaction::{
        TransactionArgument, TransactionStatus, MAX_TRANSACTION_SIZE_IN_BYTES, SCRIPT_HASH_LENGTH,
    },
    vm_error::{
        ExecutionStatus, VMStatus, VMValidationStatus, VMVerificationError, VMVerificationStatus,
    },
};
use vm::gas_schedule::{self, GasAlgebra};
use vm_genesis::encode_transfer_program;

#[test]
fn verify_signature() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 10);
    executor.add_account_data(&sender);
    // Generate a new key pair to try and sign things with.
    let other_keypair = KeyPair::new(::crypto::signing::generate_keypair().0);
    let program = encode_transfer_program(sender.address(), 100);
    let signed_txn = transaction_test_helpers::get_test_unchecked_txn(
        *sender.address(),
        0,
        other_keypair.private_key().clone(),
        sender.account().pubkey,
        Some(program),
    );

    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()),
        executor.execute_transaction(signed_txn).status(),
        VMStatus::Validation(VMValidationStatus::InvalidSignature)
    );
}

#[test]
fn verify_rejected_write_set() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 10);
    executor.add_account_data(&sender);
    let signed_txn = transaction_test_helpers::get_write_set_txn(
        *sender.address(),
        0,
        sender.account().privkey.clone(),
        sender.account().pubkey,
        None,
    )
    .into_inner();

    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()),
        executor.execute_transaction(signed_txn).status(),
        VMStatus::Validation(VMValidationStatus::RejectedWriteSet)
    );
}

#[test]
fn verify_whitelist() {
    // Making sure the whitelist's hash matches the current compiled script. If this fails, please
    // try run `cargo run` under vm_genesis and update the vm_config in node.config.toml and in
    // config.rs in libra/config crate.
    let programs: HashSet<_> = vec![
        PEER_TO_PEER.clone(),
        MINT.clone(),
        ROTATE_KEY.clone(),
        CREATE_ACCOUNT.clone(),
    ]
    .into_iter()
    .map(|s| {
        let mut hash = [0u8; SCRIPT_HASH_LENGTH];
        let mut keccak = Keccak::new_sha3_256();

        keccak.update(&s);
        keccak.finalize(&mut hash);
        hash
    })
    .collect();

    let config = NodeConfigHelpers::get_single_node_test_config(false);

    assert_eq!(
        Some(&programs),
        config.vm_config.publishing_options.get_whitelist_set()
    )
}

#[test]
fn verify_simple_payment() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
    let sender = AccountData::new(900_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    // define the arguments to the peer to peer transaction
    let transfer_amount = 1_000;
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(transfer_amount));

    // Create a new transaction that has the exact right sequence number.
    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10, // this should be programmable but for now is 1 more than the setup
        100_000,
        1,
    );
    assert_eq!(executor.verify_transaction(txn), None);

    // Create a new transaction that has the bad auth key.
    let txn = sender.account().create_signed_txn_with_args_and_sender(
        *receiver.address(),
        PEER_TO_PEER.clone(),
        args.clone(),
        10, // this should be programmable but for now is 1 more than the setup
        100_000,
        1,
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::InvalidAuthKey)
    );

    // Create a new transaction that has a old sequence number.
    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        1,
        100_000,
        1,
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::SequenceNumberTooOld)
    );

    // Create a new transaction that has a too new sequence number.
    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        11,
        100_000,
        1,
    );
    assert_prologue_disparity!(
        executor.verify_transaction(txn.clone()) => None,
        executor.execute_transaction(txn).status() =>
        TransactionStatus::Discard(VMStatus::Validation(
                VMValidationStatus::SequenceNumberTooNew
        ))
    );

    // Create a new transaction that doesn't have enough balance to pay for gas.
    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10,
        1_000_000,
        1,
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::InsufficientBalanceForTransactionFee)
    );

    // XXX TZ: TransactionExpired

    // RejectedWriteSet is tested in `verify_rejected_write_set`
    // InvalidWriteSet is tested in genesis.rs

    // Create a new transaction from a bogus account that doesn't exist
    let bogus_account = AccountData::new(100_000, 10);
    let txn = bogus_account.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10,
        10_000,
        1,
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::SendingAccountDoesNotExist(_))
    );

    // RejectedWriteSet is tested in `verify_rejected_write_set`
    // InvalidWriteSet is tested in genesis.rs

    // The next couple tests test transaction size, and bounds on gas price and the number of gas
    // units that can be submitted with a transaction.
    //
    // We test these in the reverse order that they appear in verify_transaction, and build up the
    // errors one-by-one to make sure that we are both catching all of them, and that we are doing
    // so in the specified order.
    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10,
        1_000_000,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get() + 1,
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::GasUnitPriceAboveMaxBound(_))
    );

    // Note: We can't test this at the moment since MIN_PRICE_PER_GAS_UNIT is set to 0 for testnet.
    // Uncomment this test once we have a non-zero MIN_PRICE_PER_GAS_UNIT.
    // let txn = sender.account().create_signed_txn_with_args(
    //     PEER_TO_PEER.clone(),
    //     args.clone(),
    //     10,
    //     1_000_000,
    //     gas_schedule::MIN_PRICE_PER_GAS_UNIT - 1,
    // );
    // assert_eq!(
    //     executor.verify_transaction(txn),
    //     Some(VMStatus::Validation(
    //         VMValidationStatus::GasUnitPriceBelowMinBound
    //     ))
    // );

    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10,
        1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(_))
    );

    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10,
        gas_schedule::MIN_TRANSACTION_GAS_UNITS.get() - 1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(_))
    );

    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args.clone(),
        10,
        gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get() + 1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::MaxGasUnitsExceedsMaxGasUnitsBound(_))
    );

    let txn = sender.account().create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        vec![TransactionArgument::U64(42); MAX_TRANSACTION_SIZE_IN_BYTES],
        10,
        gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get() + 1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::ExceededMaxTransactionSize(_))
    );

    // Create a new transaction that swaps the two arguments.
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::U64(transfer_amount));
    args.push(TransactionArgument::Address(*receiver.address()));

    let txn =
        sender
            .account()
            .create_signed_txn_with_args(PEER_TO_PEER.clone(), args, 10, 100_000, 1);
    assert_eq!(
        executor.verify_transaction(txn),
        Some(VMStatus::Verification(vec![VMVerificationStatus::Script(
            VMVerificationError::TypeMismatch("Actual Type Mismatch".to_string())
        )]))
    );

    // Create a new transaction that has no argument.
    let txn =
        sender
            .account()
            .create_signed_txn_with_args(PEER_TO_PEER.clone(), vec![], 10, 100_000, 1);
    assert_eq!(
        executor.verify_transaction(txn),
        Some(VMStatus::Verification(vec![VMVerificationStatus::Script(
            VMVerificationError::TypeMismatch("Actual Type Mismatch".to_string())
        )]))
    );
}

#[test]
pub fn test_whitelist() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    // create an empty transaction
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let random_script = compile_script("main() {return;}");
    let txn = sender
        .account()
        .create_signed_txn_with_args(random_script, vec![], 10, 100_000, 1);
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::UnknownScript)
    );
}

#[test]
pub fn test_arbitrary_script_execution() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::CustomScripts);

    // create an empty transaction
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let random_script = compile_script("main() {return;}");
    let txn = sender
        .account()
        .create_signed_txn_with_args(random_script, vec![], 10, 100_000, 1);
    assert_eq!(executor.verify_transaction(txn.clone()), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
    );
}

#[test]
pub fn test_no_publishing() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::CustomScripts);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let program = String::from(
        "
        modules:
        module M {
            public max(a: u64, b: u64): u64 {
                if (copy(a) > copy(b)) {
                    return copy(a);
                } else {
                    return copy(b);
                }
                return 0;
            }

            public sum(a: u64, b: u64): u64 {
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }
        }
        script:
        import 0x0.LibraAccount;
        main (payee: address, amount: u64) {
          LibraAccount.pay_from_sender(move(payee), move(amount));
          return;
        }
        ",
    );

    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(100));

    let random_script = compile_program_with_address(sender.address(), &program, args);
    let txn =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 10, 100_000, 1);
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::UnknownModule)
    );
}

#[test]
pub fn test_open_publishing_invalid_address() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let program = String::from(
        "
        modules:
        module M {
            public max(a: u64, b: u64): u64 {
                if (copy(a) > copy(b)) {
                    return copy(a);
                } else {
                    return copy(b);
                }
                return 0;
            }

            public sum(a: u64, b: u64): u64 {
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }
        }
        script:
        import 0x0.LibraAccount;
        main (payee: address, amount: u64) {
          LibraAccount.pay_from_sender(move(payee), move(amount));
          return;
        }
        ",
    );

    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(100));

    let random_script = compile_program_with_address(receiver.address(), &program, args);
    let txn =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 10, 100_000, 1);

    // verify and fail because the addresses don't match
    let vm_status = executor.verify_transaction(txn.clone());
    let status = match vm_status {
        Some(VMStatus::Verification(status)) => status,
        vm_status => panic!("Unexpected verification status: {:?}", vm_status),
    };
    match status.as_slice() {
        &[VMVerificationStatus::Module(
            0,
            VMVerificationError::ModuleAddressDoesNotMatchSender(_),
        )] => {}
        err => panic!("Unexpected verification error: {:?}", err),
    };

    // execute and fail for the same reason
    let output = executor.execute_transaction(txn);
    let status = match output.status() {
        TransactionStatus::Discard(VMStatus::Verification(status)) => status,
        vm_status => panic!("Unexpected verification status: {:?}", vm_status),
    };
    match status.as_slice() {
        &[VMVerificationStatus::Module(
            0,
            VMVerificationError::ModuleAddressDoesNotMatchSender(_),
        )] => {}
        err => panic!("Unexpected verification error: {:?}", err),
    };
}

#[test]
pub fn test_open_publishing() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let program = String::from(
        "
        modules:
        module M {
            public max(a: u64, b: u64): u64 {
                if (copy(a) > copy(b)) {
                    return copy(a);
                } else {
                    return copy(b);
                }
                return 0;
            }

            public sum(a: u64, b: u64): u64 {
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }
        }
        script:
        import 0x0.LibraAccount;
        main (payee: address, amount: u64) {
          LibraAccount.pay_from_sender(move(payee), move(amount));
          return;
        }
        ",
    );

    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(100));

    let random_script = compile_program_with_address(sender.address(), &program, args);
    let txn =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 10, 100_000, 1);
    assert_eq!(executor.verify_transaction(txn.clone()), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
    );
}

#[test]
fn test_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // Get a module that fails verification into the store.
    let bad_module_code = "
    modules:
    module Test {
        resource R1 { }
        struct S1 { r1: R#Self.R1 }

        public new_S1(): V#Self.S1 {
            let s: V#Self.S1;
            let r: R#Self.R1;
            r = R1 {};
            s = S1 { r1: move(r) };
            return move(s);
        }
    }

    script:
    main() {
    }
    ";
    let compiler = Compiler {
        code: bad_module_code,
        ..Compiler::default()
    };
    let mut modules = compiler
        .into_compiled_program()
        .expect("Failed to compile")
        .modules;
    let module = modules.swap_remove(0);
    executor.add_module(&module.self_id(), &module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    import 0x0.Test;

    main() {
        let x: V#Test.S1;
        x = Test.new_S1();
        return;
    }
    ";

    let compiler = Compiler {
        code,
        address: *sender.address(),
        // This is OK because we *know* the module is unverified.
        extra_deps: vec![VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(
            module,
        )],
        ..Compiler::default()
    };
    let program = compiler.into_program(vec![]).expect("Failed to compile");
    let txn = sender
        .account()
        .create_signed_txn_impl(*sender.address(), program, 10, 100_000, 1);
    // As of now, we don't verify dependencies in verify_transaction.
    assert_eq!(executor.verify_transaction(txn.clone()), None);
    let errors = match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(VMStatus::Verification(errors)) => errors.to_vec(),
        other => panic!("Unexpected status: {:?}", other),
    };
    assert_matches!(
        &errors[0],
        VMVerificationStatus::Dependency(module_id, _)
            if module_id.address() == &AccountAddress::default() && module_id.name() == "Test"
    );
}
