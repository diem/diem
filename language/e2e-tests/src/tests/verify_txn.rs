// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountData, assert_prologue_disparity, assert_prologue_parity, assert_status_eq,
    compile::compile_module_with_address, executor::FakeExecutor, transaction_status_eq,
};
use bytecode_verifier::VerifiedModule;
use compiler::Compiler;
use libra_crypto::{ed25519::Ed25519PrivateKey, TPrivateKey, Uniform};
use libra_types::{
    account_config::{lbr_type_tag, CORE_CODE_ADDRESS, LBR_NAME},
    on_chain_config::VMPublishingOption,
    test_helpers::transaction_test_helpers,
    transaction::{
        Script, TransactionArgument, TransactionPayload, TransactionStatus,
        MAX_TRANSACTION_SIZE_IN_BYTES,
    },
    vm_error::{StatusCode, StatusType, VMStatus},
};
use stdlib::transaction_scripts::StdlibScript;
use transaction_builder::encode_transfer_with_metadata_script;
use vm::gas_schedule::{self, GasAlgebra};

#[test]
fn verify_signature() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 10);
    executor.add_account_data(&sender);
    // Generate a new key pair to try and sign things with.
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let program =
        encode_transfer_with_metadata_script(lbr_type_tag(), sender.address(), vec![], 100, vec![]);
    let signed_txn = transaction_test_helpers::get_test_unchecked_txn(
        *sender.address(),
        0,
        &private_key,
        sender.account().pubkey.clone(),
        Some(program),
    );

    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        VMStatus::new(StatusCode::INVALID_SIGNATURE)
    );
}

#[test]
fn verify_reserved_sender() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 10);
    executor.add_account_data(&sender);
    // Generate a new key pair to try and sign things with.
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let program =
        encode_transfer_with_metadata_script(lbr_type_tag(), sender.address(), vec![], 100, vec![]);
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        CORE_CODE_ADDRESS,
        0,
        &private_key,
        private_key.public_key(),
        Some(program),
    );

    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST)
    );
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

    let p2p_script = StdlibScript::PeerToPeer.compiled_bytes().into_vec();

    // Create a new transaction that has the exact right sequence number.
    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10, // this should be programmable but for now is 1 more than the setup
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_eq!(executor.verify_transaction(txn).status(), None);

    // Create a new transaction that has the bad auth key.
    let txn = sender.account().create_signed_txn_with_args_and_sender(
        *receiver.address(),
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10, // this should be programmable but for now is 1 more than the setup
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::INVALID_AUTH_KEY)
    );

    // Create a new transaction that has a old sequence number.
    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        1,
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD)
    );

    // Create a new transaction that has a too new sequence number.
    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        11,
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_prologue_disparity!(
        executor.verify_transaction(txn.clone()).status() => None,
        executor.execute_transaction(txn).status() =>
        TransactionStatus::Discard(VMStatus::new(
                StatusCode::SEQUENCE_NUMBER_TOO_NEW
        ))
    );

    // Create a new transaction that doesn't have enough balance to pay for gas.
    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10,
        1_000_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE)
    );

    // XXX TZ: TransactionExpired

    // RejectedWriteSet is tested in `verify_rejected_write_set`
    // InvalidWriteSet is tested in genesis.rs

    // Create a new transaction from a bogus account that doesn't exist
    let bogus_account = AccountData::new(100_000, 10);
    let txn = bogus_account.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10,
        10_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST)
    );

    // RejectedWriteSet is tested in `verify_rejected_write_set`
    // InvalidWriteSet is tested in genesis.rs

    // The next couple tests test transaction size, and bounds on gas price and the number of
    // gas units that can be submitted with a transaction.
    //
    // We test these in the reverse order that they appear in verify_transaction, and build up
    // the errors one-by-one to make sure that we are both catching all of them, and
    // that we are doing so in the specified order.
    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10,
        1_000_000,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get() + 1,
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND)
    );

    // Note: We can't test this at the moment since MIN_PRICE_PER_GAS_UNIT is set to 0 for
    // testnet. Uncomment this test once we have a non-zero MIN_PRICE_PER_GAS_UNIT.
    // let txn = sender.account().create_signed_txn_with_args(
    //     p2p_script.clone(),
    //     args.clone(),
    //     10,
    //     1_000_000,
    //     gas_schedule::MIN_PRICE_PER_GAS_UNIT - 1,
    // );
    // assert_eq!(
    //     executor.verify_transaction(txn).status(),
    //     Some(VMStatus::new(
    //         StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND
    //     ))
    // );

    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10,
        1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS)
    );

    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args.clone(),
        10,
        gas_schedule::MIN_TRANSACTION_GAS_UNITS.get() - 1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS)
    );

    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args,
        10,
        gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get() + 1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND)
    );

    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        vec![TransactionArgument::U64(42); MAX_TRANSACTION_SIZE_IN_BYTES],
        10,
        gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get() + 1,
        gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
        LBR_NAME.to_string(),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE)
    );

    // Create a new transaction that swaps the two arguments.
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::U64(transfer_amount));
    args.push(TransactionArgument::Address(*receiver.address()));

    let txn = sender.account().create_signed_txn_with_args(
        p2p_script.clone(),
        vec![lbr_type_tag()],
        args,
        10,
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(
            VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("argument length mismatch: expected 3 got 2".to_string())
        )
    );

    // Create a new transaction that has no argument.
    let txn = sender.account().create_signed_txn_with_args(
        p2p_script,
        vec![lbr_type_tag()],
        vec![],
        10,
        100_000,
        1,
        LBR_NAME.to_string(),
    );

    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(
            VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("argument length mismatch: expected 3 got 0".to_string())
        )
    );
}

#[test]
pub fn test_whitelist() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::whitelist_genesis();
    // create an empty transaction
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // When CustomScripts is off, a garbage script should be rejected with Keep(UnknownScript)
    let random_script = vec![];
    let txn = sender.account().create_signed_txn_with_args(
        random_script,
        vec![],
        vec![],
        10,
        100_000,
        1,
        LBR_NAME.to_string(),
    );

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::UNKNOWN_SCRIPT)
    );
}

#[test]
pub fn test_arbitrary_script_execution() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::CustomScripts);

    // create an empty transaction
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // If CustomScripts is on, result should be Keep(DeserializationError). If it's off, the
    // result should be Keep(UnknownScript)
    let random_script = vec![];
    let txn = sender.account().create_signed_txn_with_args(
        random_script,
        vec![],
        vec![],
        10,
        100_000,
        1,
        LBR_NAME.to_string(),
    );

    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    let status = executor.execute_transaction(txn).status().clone();
    assert!(!status.is_discarded());
    assert_eq!(
        status.vm_status().major_status,
        StatusCode::CODE_DESERIALIZATION_ERROR,
    );
}

#[test]
pub fn test_no_publishing() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::CustomScripts);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let module = String::from(
        "
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
        ",
    );

    let random_module = compile_module_with_address(sender.address(), "file_name", &module);
    let txn = sender
        .account()
        .create_user_txn(random_module, 10, 100_000, 1, LBR_NAME.to_string());
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::UNKNOWN_MODULE)
    );
}

#[test]
pub fn test_open_publishing_invalid_address() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let module = String::from(
        "
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
        ",
    );

    let random_module = compile_module_with_address(receiver.address(), "file_name", &module);
    let txn = sender
        .account()
        .create_user_txn(random_module, 10, 100_000, 1, LBR_NAME.to_string());

    // TODO: This is not verified for now.
    // verify and fail because the addresses don't match
    // let vm_status = executor.verify_transaction(txn.clone()).status().unwrap();

    // assert!(vm_status.is(StatusType::Verification));
    // assert!(vm_status.major_status == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER);

    // execute and fail for the same reason
    let output = executor.execute_transaction(txn);
    if let TransactionStatus::Keep(status) = output.status() {
        assert!(status.major_status == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER)
    } else {
        panic!("Unexpected execution status: {:?}", output)
    };
}

#[test]
pub fn test_open_publishing() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
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
        ",
    );

    let random_module = compile_module_with_address(sender.address(), "file_name", &program);
    let txn = sender
        .account()
        .create_user_txn(random_module, 10, 100_000, 1, LBR_NAME.to_string());
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
}

#[test]
fn test_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // Get a module that fails verification into the store.
    let bad_module_code = "
    module Test {
        resource R1 { b: bool }
        struct S1 { r1: Self.R1 }

        public new_S1(): Self.S1 {
            let s: Self.S1;
            let r: Self.R1;
            r = R1 { b: true };
            s = S1 { r1: move(r) };
            return move(s);
        }
    }
    ";
    let compiler = Compiler {
        ..Compiler::default()
    };
    let module = compiler
        .into_compiled_module("file_name", bad_module_code)
        .expect("Failed to compile");
    executor.add_module(&module.self_id(), &module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    import 0x0.Test;

    main() {
        let x: Test.S1;
        x = Test.new_S1();
        return;
    }
    ";

    let compiler = Compiler {
        address: *sender.address(),
        // This is OK because we *know* the module is unverified.
        extra_deps: vec![VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(
            module,
        )],
        ..Compiler::default()
    };
    let script = compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile");
    let txn = sender.account().create_user_txn(
        TransactionPayload::Script(Script::new(script, vec![], vec![])),
        10,
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    // As of now, we don't verify dependencies in verify_transaction.
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Keep(status) => {
            assert!(status.is(StatusType::Verification));
            assert!(status.major_status == StatusCode::INVALID_RESOURCE_FIELD);
        }
        _ => panic!("Failed to find missing dependency in bytecode verifier"),
    }
}
