// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::transaction_scripts::StdlibScript;
use compiler::Compiler;
use language_e2e_tests::{
    account::{Account, AccountData},
    assert_prologue_disparity, assert_prologue_parity,
    compile::compile_module_with_address,
    executor::FakeExecutor,
    gas_costs, transaction_status_eq,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::{
    account_address::AccountAddress,
    account_config::{self, lbr_type_tag},
    chain_id::ChainId,
    on_chain_config::VMPublishingOption,
    test_helpers::transaction_test_helpers,
    transaction::{Script, TransactionArgument, TransactionStatus, MAX_TRANSACTION_SIZE_IN_BYTES},
    vm_status::{KeptVMStatus, StatusCode},
};
use move_core_types::{
    gas_schedule::{GasAlgebra, GasConstants},
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use transaction_builder::encode_peer_to_peer_with_metadata_script;
use vm::file_format::CompiledModule;

#[test]
fn verify_signature() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 10);
    executor.add_account_data(&sender);
    // Generate a new key pair to try and sign things with.
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let program = encode_peer_to_peer_with_metadata_script(
        lbr_type_tag(),
        *sender.address(),
        100,
        vec![],
        vec![],
    );
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
        StatusCode::INVALID_SIGNATURE
    );
}

#[test]
fn verify_reserved_sender() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 10);
    executor.add_account_data(&sender);
    // Generate a new key pair to try and sign things with.
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let program = encode_peer_to_peer_with_metadata_script(
        lbr_type_tag(),
        *sender.address(),
        100,
        vec![],
        vec![],
    );
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        account_config::reserved_vm_address(),
        0,
        &private_key,
        private_key.public_key(),
        Some(program),
    );

    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
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
    args.push(TransactionArgument::U8Vector(vec![]));
    args.push(TransactionArgument::U8Vector(vec![]));

    let p2p_script = StdlibScript::PeerToPeerWithMetadata
        .compiled_bytes()
        .into_vec();

    // Create a new transaction that has the exact right sequence number.
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .sign();
    assert_eq!(executor.verify_transaction(txn).status(), None);

    // Create a new transaction that has the bad auth key.
    let txn = receiver
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .raw()
        .sign(&sender.account().privkey, sender.account().pubkey.clone())
        .unwrap()
        .into_inner();

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INVALID_AUTH_KEY
    );

    // Create a new transaction that has a old sequence number.
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(1)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::SEQUENCE_NUMBER_TOO_OLD
    );

    // Create a new transaction that has a too new sequence number.
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(11)
        .sign();
    assert_prologue_disparity!(
        executor.verify_transaction(txn.clone()).status() => None,
        executor.execute_transaction(txn).status() =>
        TransactionStatus::Discard(StatusCode::SEQUENCE_NUMBER_TOO_NEW)
    );

    // Create a new transaction that doesn't have enough balance to pay for gas.
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .max_gas_amount(1_000_000)
        .gas_unit_price(1)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE
    );

    // Create a new transaction from a bogus account that doesn't exist
    let bogus_account = AccountData::new(100_000, 10);
    let txn = bogus_account
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
    );

    // The next couple tests test transaction size, and bounds on gas price and the number of
    // gas units that can be submitted with a transaction.
    //
    // We test these in the reverse order that they appear in verify_transaction, and build up
    // the errors one-by-one to make sure that we are both catching all of them, and
    // that we are doing so in the specified order.
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .gas_unit_price(GasConstants::default().max_price_per_gas_unit.get() + 1)
        .max_gas_amount(1_000_000)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND
    );

    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .max_gas_amount(1)
        .gas_unit_price(GasConstants::default().max_price_per_gas_unit.get())
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS
    );

    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            args.clone(),
        ))
        .sequence_number(10)
        .max_gas_amount(GasConstants::default().min_transaction_gas_units.get() - 1)
        .gas_unit_price(GasConstants::default().max_price_per_gas_unit.get())
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS
    );

    let txn = sender
        .account()
        .transaction()
        .script(Script::new(p2p_script.clone(), vec![lbr_type_tag()], args))
        .sequence_number(10)
        .max_gas_amount(GasConstants::default().maximum_number_of_gas_units.get() + 1)
        .gas_unit_price(GasConstants::default().max_price_per_gas_unit.get())
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND
    );

    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            p2p_script.clone(),
            vec![lbr_type_tag()],
            vec![TransactionArgument::U64(42); MAX_TRANSACTION_SIZE_IN_BYTES],
        ))
        .sequence_number(10)
        .max_gas_amount(GasConstants::default().maximum_number_of_gas_units.get() + 1)
        .gas_unit_price(GasConstants::default().max_price_per_gas_unit.get())
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE
    );

    // Create a new transaction that swaps the two arguments.
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::U64(transfer_amount));
    args.push(TransactionArgument::Address(*receiver.address()));

    let txn = sender
        .account()
        .transaction()
        .script(Script::new(p2p_script.clone(), vec![lbr_type_tag()], args))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    assert_eq!(
        executor.execute_transaction(txn).status(),
        // StatusCode::TYPE_MISMATCH
        &TransactionStatus::Keep(KeptVMStatus::MiscellaneousError)
    );

    // Create a new transaction that has no argument.
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(p2p_script, vec![lbr_type_tag()], vec![]))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    assert_eq!(
        executor.execute_transaction(txn).status(),
        // StatusCode::TYPE_MISMATCH
        &TransactionStatus::Keep(KeptVMStatus::MiscellaneousError)
    );
}

#[test]
pub fn test_allowlist() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    // create an empty transaction
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // When CustomScripts is off, a garbage script should be rejected with Keep(UnknownScript)
    let random_script = vec![];
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(random_script, vec![], vec![]))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::UNKNOWN_SCRIPT
    );
}

#[test]
pub fn test_arbitrary_script_execution() {
    // create a FakeExecutor with a genesis from file
    let mut executor =
        FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());

    // create an empty transaction
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // If CustomScripts is on, result should be Keep(DeserializationError). If it's off, the
    // result should be Keep(UnknownScript)
    let random_script = vec![];
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(random_script, vec![], vec![]))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    let status = executor.execute_transaction(txn).status().clone();
    assert!(!status.is_discarded());
    assert_eq!(
        status.status(),
        // StatusCode::CODE_DESERIALIZATION_ERROR
        Ok(KeptVMStatus::MiscellaneousError)
    );
}

#[test]
pub fn test_publish_from_libra_root() {
    // create a FakeExecutor with a genesis from file
    let mut executor =
        FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());

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
        .transaction()
        .module(random_module)
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INVALID_MODULE_PUBLISHER
    );
}

#[test]
fn verify_expiration_time() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 0);
    executor.add_account_data(&sender);
    let private_key = &sender.account().privkey;
    let txn = transaction_test_helpers::get_test_signed_transaction(
        *sender.address(),
        0, /* sequence_number */
        private_key,
        private_key.public_key(),
        None, /* script */
        0,    /* expiration_time */
        0,    /* gas_unit_price */
        account_config::LBR_NAME.to_owned(),
        None, /* max_gas_amount */
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::TRANSACTION_EXPIRED
    );
}

#[test]
fn verify_chain_id() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 0);
    executor.add_account_data(&sender);
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let txn = transaction_test_helpers::get_test_txn_with_chain_id(
        *sender.address(),
        0,
        &private_key,
        private_key.public_key(),
        // all tests use ChainId::test() for chain_id,so pick something different
        ChainId::new(ChainId::test().id() + 1),
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::BAD_CHAIN_ID
    );
}

#[test]
fn verify_gas_currency_with_bad_identifier() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 0);
    executor.add_account_data(&sender);
    let private_key = &sender.account().privkey;
    let txn = transaction_test_helpers::get_test_signed_transaction(
        *sender.address(),
        0, /* sequence_number */
        private_key,
        private_key.public_key(),
        None,     /* script */
        u64::MAX, /* expiration_time */
        0,        /* gas_unit_price */
        // The gas currency code is treated as a Move identifier, so it needs to follow
        // the rules about starting with a letter or underscore and containing only
        // alphanumeric and underscore characters.
        "1BadID".to_string(),
        None, /* max_gas_amount */
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INVALID_GAS_SPECIFIER
    );
}

#[test]
fn verify_gas_currency_code() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(900_000, 0);
    executor.add_account_data(&sender);
    let private_key = &sender.account().privkey;
    let txn = transaction_test_helpers::get_test_signed_transaction(
        *sender.address(),
        0, /* sequence_number */
        private_key,
        private_key.public_key(),
        None,     /* script */
        u64::MAX, /* expiration_time */
        0,        /* gas_unit_price */
        "INVALID".to_string(),
        None, /* max_gas_amount */
    );
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::CURRENCY_INFO_DOES_NOT_EXIST
    );
}

#[test]
pub fn test_no_publishing_libra_root_sender() {
    // create a FakeExecutor with a genesis from file
    let executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());

    // create a transaction trying to publish a new module.
    let sender = Account::new_libra_root();

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

    let random_module =
        compile_module_with_address(&account_config::CORE_CODE_ADDRESS, "file_name", &module);
    let txn = sender
        .transaction()
        .module(random_module)
        .sequence_number(1)
        .max_gas_amount(100_000)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}

#[test]
pub fn test_open_publishing_invalid_address() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

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
        .transaction()
        .module(random_module)
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();

    // TODO: This is not verified for now.
    // verify and fail because the addresses don't match
    // let vm_status = executor.verify_transaction(txn.clone()).status().unwrap();

    // assert!(vm_status.is(StatusType::Verification));
    // assert!(vm_status.major_status == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER);

    // execute and fail for the same reason
    let output = executor.execute_transaction(txn);
    if let TransactionStatus::Keep(status) = output.status() {
        // assert!(status.status_code() == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER)
        assert!(status == &KeptVMStatus::MiscellaneousError);
    } else {
        panic!("Unexpected execution status: {:?}", output)
    };
}

#[test]
pub fn test_open_publishing() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

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
        .transaction()
        .module(random_module)
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}

fn bad_module() -> CompiledModule {
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
    compiler
        .into_compiled_module("file_name", bad_module_code)
        .expect("Failed to compile")
}

fn good_module_uses_bad(address: AccountAddress, bad_dep: CompiledModule) -> CompiledModule {
    let good_module_code = "
    module Test2 {
        import 0x1.Test;
        struct S { b: bool }

        foo(): Test.S1 {
            return Test.new_S1();
        }
        public bar() {
            return;
        }
    }
    ";

    let compiler = Compiler {
        address,
        extra_deps: vec![bad_dep],
        ..Compiler::default()
    };
    compiler
        .into_compiled_module("file_name", good_module_code)
        .expect("Failed to compile")
}

#[test]
fn test_script_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

    // Get a module that fails verification into the store.
    let module = bad_module();
    executor.add_module(&module.self_id(), &module);

    // Create a module that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    import 0x1.Test;

    main() {
        let x: Test.S1;
        x = Test.new_S1();
        return;
    }
    ";

    let compiler = Compiler {
        address: *sender.address(),
        // This is OK because we *know* the module is unverified.
        extra_deps: vec![module],
        ..Compiler::default()
    };
    let script = compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile");
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(script, vec![], vec![]))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    // As of now, we verify module/script dependencies. This will result in an
    // invariant violation as we try to load `Test`
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(status) => {
            assert_eq!(status, &StatusCode::UNEXPECTED_VERIFIER_ERROR);
        }
        _ => panic!("Kept transaction with an invariant violation!"),
    }
}

#[test]
fn test_module_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

    // Get a module that fails verification into the store.
    let bad_module = bad_module();
    executor.add_module(&bad_module.self_id(), &bad_module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    let good_module = {
        let m = good_module_uses_bad(*sender.address(), bad_module);
        let mut serialized_module = Vec::<u8>::new();
        m.serialize(&mut serialized_module).unwrap();
        libra_types::transaction::Module::new(serialized_module)
    };

    let txn = sender
        .account()
        .transaction()
        .module(good_module)
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    // As of now, we verify module/script dependencies. This will result in an
    // invariant violation as we try to load `Test`
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(status) => {
            assert_eq!(status, &StatusCode::UNEXPECTED_VERIFIER_ERROR);
        }
        _ => panic!("Kept transaction with an invariant violation!"),
    }
}

#[test]
fn test_type_tag_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

    // Get a module that fails verification into the store.
    let module = bad_module();
    executor.add_module(&module.self_id(), &module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    main<T>() {
        return;
    }
    ";

    let compiler = Compiler {
        address: *sender.address(),
        // This is OK because we *know* the module is unverified.
        extra_deps: vec![module],
        ..Compiler::default()
    };
    let script = compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile");
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            script,
            vec![TypeTag::Struct(StructTag {
                address: account_config::CORE_CODE_ADDRESS,
                module: Identifier::new("Test").unwrap(),
                name: Identifier::new("S1").unwrap(),
                type_params: vec![],
            })],
            vec![],
        ))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    // As of now, we verify module/script dependencies. This will result in an
    // invariant violation as we try to load `Test`
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(status) => {
            assert_eq!(status, &StatusCode::UNEXPECTED_VERIFIER_ERROR);
        }
        _ => panic!("Kept transaction with an invariant violation!"),
    }
}

#[test]
fn test_script_transitive_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

    // Get a module that fails verification into the store.
    let bad_module = bad_module();
    executor.add_module(&bad_module.self_id(), &bad_module);

    // Create a module that tries to use that module.
    let good_module = good_module_uses_bad(account_config::CORE_CODE_ADDRESS, bad_module);
    executor.add_module(&good_module.self_id(), &good_module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    import 0x1.Test2;

    main() {
        Test2.bar();
        return;
    }
    ";

    let compiler = Compiler {
        address: *sender.address(),
        // This is OK because we *know* the module is unverified.
        extra_deps: vec![good_module],
        ..Compiler::default()
    };
    let script = compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile");
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(script, vec![], vec![]))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    // As of now, we verify module/script dependencies. This will result in an
    // invariant violation as we try to load `Test`
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(status) => {
            assert_eq!(status, &StatusCode::UNEXPECTED_VERIFIER_ERROR);
        }
        _ => panic!("Kept transaction with an invariant violation!"),
    }
}

#[test]
fn test_module_transitive_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

    // Get a module that fails verification into the store.
    let bad_module = bad_module();
    executor.add_module(&bad_module.self_id(), &bad_module);

    // Create a module that tries to use that module.
    let good_module = good_module_uses_bad(account_config::CORE_CODE_ADDRESS, bad_module);
    executor.add_module(&good_module.self_id(), &good_module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let module_code = "
    module Test3 {
        import 0x1.Test2;
        public bar() {
            Test2.bar();
            return;
        }
    }
    ";
    let module = {
        let compiler = Compiler {
            address: *sender.address(),
            extra_deps: vec![good_module],
            ..Compiler::default()
        };
        libra_types::transaction::Module::new(
            compiler
                .into_module_blob("file_name", module_code)
                .expect("Module compilation failed"),
        )
    };

    let txn = sender
        .account()
        .transaction()
        .module(module)
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    // As of now, we verify module/script dependencies. This will result in an
    // invariant violation as we try to load `Test`
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(status) => {
            assert_eq!(status, &StatusCode::UNEXPECTED_VERIFIER_ERROR);
        }
        _ => panic!("Kept transaction with an invariant violation!"),
    }
}

#[test]
fn test_type_tag_transitive_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());

    // Get a module that fails verification into the store.
    let bad_module = bad_module();
    executor.add_module(&bad_module.self_id(), &bad_module);

    // Create a module that tries to use that module.
    let good_module = good_module_uses_bad(account_config::CORE_CODE_ADDRESS, bad_module);
    executor.add_module(&good_module.self_id(), &good_module);

    // Create a transaction that tries to use that module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    main<T>() {
        return;
    }
    ";

    let compiler = Compiler {
        address: *sender.address(),
        // This is OK because we *know* the module is unverified.
        extra_deps: vec![good_module],
        ..Compiler::default()
    };
    let script = compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile");
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            script,
            vec![TypeTag::Struct(StructTag {
                address: account_config::CORE_CODE_ADDRESS,
                module: Identifier::new("Test2").unwrap(),
                name: Identifier::new("S").unwrap(),
                type_params: vec![],
            })],
            vec![],
        ))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();
    // As of now, we verify module/script dependencies. This will result in an
    // invariant violation as we try to load `Test`
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    match executor.execute_transaction(txn).status() {
        TransactionStatus::Discard(status) => {
            assert_eq!(status, &StatusCode::UNEXPECTED_VERIFIER_ERROR);
        }
        _ => panic!("Kept transaction with an invariant violation!"),
    }
}

#[test]
fn charge_gas_invalid_args() {
    let mut fake_executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 0);
    fake_executor.add_account_data(&sender);

    // get a SignedTransaction
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(
            StdlibScript::PeerToPeerWithMetadata
                .compiled_bytes()
                .into_vec(),
            vec![account_config::lbr_type_tag()],
            // Don't pass any arguments
            vec![],
        ))
        .sequence_number(0)
        .max_gas_amount(gas_costs::TXN_RESERVED)
        .sign();

    let output = fake_executor.execute_transaction(txn);
    assert!(!output.status().is_discarded());
    assert!(output.gas_used() > 0);
}
