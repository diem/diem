// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiler::Compiler;
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey},
    PrivateKey, SigningKey, Uniform,
};
use diem_framework_releases::legacy::transaction_scripts::LegacyStdlibScript;
use diem_keygen::KeyGen;
use diem_transaction_builder::{
    stdlib as transaction_builder, stdlib::encode_peer_to_peer_with_metadata_script,
};
use diem_types::{
    account_address::AccountAddress,
    account_config,
    chain_id::ChainId,
    on_chain_config::VMPublishingOption,
    test_helpers::transaction_test_helpers,
    transaction::{
        authenticator::{AccountAuthenticator, AuthenticationKey, MAX_NUM_OF_SIGS},
        RawTransactionWithData, Script, SignedTransaction, TransactionArgument, TransactionPayload,
        TransactionStatus, WriteSetPayload,
    },
    vm_status::{KeptVMStatus, StatusCode},
};
use language_e2e_tests::{
    account::Account,
    assert_prologue_disparity, assert_prologue_parity,
    common_transactions::{
        multi_agent_mint_script, multi_agent_swap_script, raw_multi_agent_swap_txn, rotate_key_txn,
    },
    compile::compile_module,
    current_function_name,
    executor::FakeExecutor,
    gas_costs, test_with_different_versions, transaction_status_eq,
    versioning::CURRENT_RELEASE_VERSIONS,
};
use move_binary_format::file_format::CompiledModule;
use move_core_types::{
    gas_schedule::{GasAlgebra, GasConstants, MAX_TRANSACTION_SIZE_IN_BYTES},
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    value::{serialize_values, MoveValue},
};

#[test]
fn verify_signature() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, 10);
        executor.add_account_data(&sender);
        // Generate a new key pair to try and sign things with.
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let program = encode_peer_to_peer_with_metadata_script(
            account_config::xus_tag(),
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
    }
}

#[test]
fn verify_multi_agent() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);

    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*secondary_signer.address()],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![&secondary_signer.account().privkey],
        vec![secondary_signer.account().pubkey.clone()],
        Some(multi_agent_swap_script(10, 10)),
    );
    assert_eq!(executor.verify_transaction(signed_txn).status(), None);
}

#[test]
fn verify_multi_agent_multiple_secondary_signers() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);
    let third_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);
    executor.add_account_data(&third_signer);

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
        Some(multi_agent_mint_script(100, 0)),
    );
    assert_eq!(executor.verify_transaction(signed_txn).status(), None);
}

#[test]
fn verify_multi_agent_invalid_sender_signature() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);

    let private_key = Ed25519PrivateKey::generate_for_testing();

    // Sign using the wrong key for the sender, and correct key for the secondary signer.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*secondary_signer.address()],
        10,
        &private_key,
        sender.account().pubkey.clone(),
        vec![&secondary_signer.account().privkey],
        vec![secondary_signer.account().pubkey.clone()],
        None,
    );
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::INVALID_SIGNATURE
    );
}

#[test]
fn verify_multi_agent_invalid_secondary_signature() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);

    let private_key = Ed25519PrivateKey::generate_for_testing();

    // Sign using the correct keys for the sender, but wrong keys for the secondary signer.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*secondary_signer.address()],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![&private_key],
        vec![secondary_signer.account().pubkey.clone()],
        None,
    );
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::INVALID_SIGNATURE
    );
}

#[test]
fn verify_multi_agent_num_sigs_exceeds() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let mut sender_seq_num = 10;
    let secondary_signer_seq_num = 100;
    let sender = executor.create_raw_account_data(1_000_010, sender_seq_num);
    let secondary_signer = executor.create_raw_account_data(100_100, secondary_signer_seq_num);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);

    // create two multisigs with `MAX_NUM_OF_SIGS/MAX_NUM_OF_SIGS` policy.
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let threshold = MAX_NUM_OF_SIGS as u8;

    let (sender_privkeys, sender_pubkeys): (Vec<Ed25519PrivateKey>, Vec<Ed25519PublicKey>) =
        (0..threshold).map(|_| keygen.generate_keypair()).unzip();
    let sender_multi_ed_public_key = MultiEd25519PublicKey::new(sender_pubkeys, threshold).unwrap();
    let sender_new_auth_key = AuthenticationKey::multi_ed25519(&sender_multi_ed_public_key);

    let (secondary_signer_privkeys, secondary_signer_pubkeys) =
        (0..threshold).map(|_| keygen.generate_keypair()).unzip();
    let secondary_signer_multi_ed_public_key =
        MultiEd25519PublicKey::new(secondary_signer_pubkeys, threshold).unwrap();
    let secondary_signer_new_auth_key =
        AuthenticationKey::multi_ed25519(&secondary_signer_multi_ed_public_key);

    // (1) rotate keys to multisigs
    let sender_output = &executor.execute_transaction(rotate_key_txn(
        sender.account(),
        sender_new_auth_key.to_vec(),
        sender_seq_num,
    ));
    assert_eq!(
        sender_output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
    executor.apply_write_set(sender_output.write_set());
    sender_seq_num += 1;

    let secondary_signer_output = &executor.execute_transaction(rotate_key_txn(
        secondary_signer.account(),
        secondary_signer_new_auth_key.to_vec(),
        secondary_signer_seq_num,
    ));
    assert_eq!(
        secondary_signer_output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
    executor.apply_write_set(secondary_signer_output.write_set());

    // (2) sign a txn with new multisig private keys
    let txn = raw_multi_agent_swap_txn(
        sender.account(),
        secondary_signer.account(),
        sender_seq_num,
        0,
        0,
    );
    let raw_txn_with_data =
        RawTransactionWithData::new_multi_agent(txn.clone(), vec![*secondary_signer.address()]);
    let sender_sig = MultiEd25519PrivateKey::new(sender_privkeys, threshold)
        .unwrap()
        .sign(&raw_txn_with_data);
    let secondary_signer_sig = MultiEd25519PrivateKey::new(secondary_signer_privkeys, threshold)
        .unwrap()
        .sign(&raw_txn_with_data);
    let signed_txn = SignedTransaction::new_multi_agent(
        txn,
        AccountAuthenticator::multi_ed25519(sender_multi_ed_public_key, sender_sig),
        vec![*secondary_signer.address()],
        vec![AccountAuthenticator::multi_ed25519(
            secondary_signer_multi_ed_public_key,
            secondary_signer_sig,
        )],
    );

    // Transaction will fail validation because the number of signatures exceeds the maximum number
    // of signatures allowed.
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::INVALID_SIGNATURE
    );
}

#[test]
fn verify_multi_agent_wrong_number_of_signer() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);
    let third_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);
    executor.add_account_data(&third_signer);

    // Number of secondary signers according is 2 but we only
    // include the signature of one of the secondary signers.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*secondary_signer.address(), *third_signer.address()],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![&secondary_signer.account().privkey],
        vec![secondary_signer.account().pubkey.clone()],
        Some(multi_agent_mint_script(10, 0)),
    );
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::SECONDARY_KEYS_ADDRESSES_COUNT_MISMATCH
    );
}

#[test]
fn verify_multi_agent_duplicate_sender() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);
    // Duplicates in signers: sender and secondary signer have the same address.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*sender.address()],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![&sender.account().privkey],
        vec![sender.account().pubkey.clone()],
        Some(multi_agent_swap_script(10, 10)),
    );
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::SIGNERS_CONTAIN_DUPLICATES
    );
}

#[test]
fn verify_multi_agent_duplicate_secondary_signer() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);
    let third_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);
    executor.add_account_data(&secondary_signer);
    executor.add_account_data(&third_signer);

    // Duplicates in secondary signers.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![
            *secondary_signer.address(),
            *third_signer.address(),
            *secondary_signer.address(),
        ],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![
            &secondary_signer.account().privkey,
            &third_signer.account().privkey,
            &secondary_signer.account().privkey,
        ],
        vec![
            secondary_signer.account().pubkey.clone(),
            third_signer.account().pubkey.clone(),
            secondary_signer.account().pubkey.clone(),
        ],
        None,
    );
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::SIGNERS_CONTAIN_DUPLICATES
    );
}

#[test]
fn verify_multi_agent_nonexistent_secondary_signer() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_010, 10);
    let secondary_signer = executor.create_raw_account_data(100_100, 100);

    executor.add_account_data(&sender);

    // Duplicates in signers: sender and secondary signer have the same address.
    let signed_txn = transaction_test_helpers::get_test_unchecked_multi_agent_txn(
        *sender.address(),
        vec![*secondary_signer.address()],
        10,
        &sender.account().privkey,
        sender.account().pubkey.clone(),
        vec![&secondary_signer.account().privkey],
        vec![secondary_signer.account().pubkey.clone()],
        Some(multi_agent_swap_script(10, 10)),
    );
    assert_prologue_parity!(
        executor.verify_transaction(signed_txn.clone()).status(),
        executor.execute_transaction(signed_txn).status(),
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
    );
}

#[test]
fn verify_reserved_sender() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, 10);
        executor.add_account_data(&sender);
        // Generate a new key pair to try and sign things with.
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let program = encode_peer_to_peer_with_metadata_script(
            account_config::xus_tag(),
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
            Some(TransactionPayload::Script(program)),
        );

        assert_prologue_parity!(
            executor.verify_transaction(signed_txn.clone()).status(),
            executor.execute_transaction(signed_txn).status(),
            StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
        );
    }
    }
}

#[test]
fn verify_simple_payment() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
        let sender = executor.create_raw_account_data(900_000, 10);
        let receiver = executor.create_raw_account_data(100_000, 10);
        executor.add_account_data(&sender);
        executor.add_account_data(&receiver);

        // define the arguments to the peer to peer transaction
        let transfer_amount = 1_000;
        let args: Vec<TransactionArgument> = vec![
            TransactionArgument::Address(*receiver.address()),
            TransactionArgument::U64(transfer_amount),
            TransactionArgument::U8Vector(vec![]),
            TransactionArgument::U8Vector(vec![]),
        ];

        let p2p_script = LegacyStdlibScript::PeerToPeerWithMetadata
            .compiled_bytes()
            .into_vec();

        // Create a new transaction that has the exact right sequence number.
        let txn = sender
            .account()
            .transaction()
            .script(Script::new(
                p2p_script.clone(),
                vec![account_config::xus_tag()],
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
                vec![account_config::xus_tag()],
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
                vec![account_config::xus_tag()],
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
                vec![account_config::xus_tag()],
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
                vec![account_config::xus_tag()],
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
        let bogus_account = executor.create_raw_account_data(100_000, 10);
        let txn = bogus_account
            .account()
            .transaction()
            .script(Script::new(
                p2p_script.clone(),
                vec![account_config::xus_tag()],
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
        let gas_constants = &GasConstants::default();

        let txn = sender
            .account()
            .transaction()
            .script(Script::new(
                p2p_script.clone(),
                vec![account_config::xus_tag()],
                args.clone(),
            ))
            .sequence_number(10)
            .gas_unit_price(gas_constants.max_price_per_gas_unit.get() + 1)
            .max_gas_amount(1_000_000)
            .sign();
        assert_prologue_parity!(
            executor.verify_transaction(txn.clone()).status(),
            executor.execute_transaction(txn).status(),
            StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND
        );

        // Test for a max_gas_amount that is insufficient to pay the minimum fee.
        // Find the minimum transaction gas units and subtract 1.
        let mut gas_limit = gas_constants
            .to_external_units(gas_constants.min_transaction_gas_units)
            .get();
        if gas_limit > 0 {
            gas_limit -= 1;
        }
        // Calculate how many extra bytes of transaction arguments to add to ensure
        // that the minimum transaction gas gets rounded up when scaling to the
        // external gas units. (Ignore the size of the script itself for simplicity.)
        let extra_txn_bytes = if gas_constants.gas_unit_scaling_factor
            > gas_constants.min_transaction_gas_units.get()
        {
            gas_constants.large_transaction_cutoff.get()
                + (gas_constants.gas_unit_scaling_factor / gas_constants.intrinsic_gas_per_byte.get())
        } else {
            0
        };
        let txn = sender
            .account()
            .transaction()
            .script(Script::new(
                p2p_script.clone(),
                vec![account_config::xus_tag()],
                vec![TransactionArgument::U8(42); extra_txn_bytes as usize],
            ))
            .sequence_number(10)
            .max_gas_amount(gas_limit)
            .gas_unit_price(gas_constants.max_price_per_gas_unit.get())
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
                vec![account_config::xus_tag()],
                args,
            ))
            .sequence_number(10)
            .max_gas_amount(gas_constants.maximum_number_of_gas_units.get() + 1)
            .gas_unit_price(gas_constants.max_price_per_gas_unit.get())
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
                vec![account_config::xus_tag()],
                vec![TransactionArgument::U8(42); MAX_TRANSACTION_SIZE_IN_BYTES as usize],
            ))
            .sequence_number(10)
            .max_gas_amount(gas_constants.maximum_number_of_gas_units.get() + 1)
            .gas_unit_price(gas_constants.max_price_per_gas_unit.get())
            .sign();
        assert_prologue_parity!(
            executor.verify_transaction(txn.clone()).status(),
            executor.execute_transaction(txn).status(),
            StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE
        );

        // Create a new transaction that swaps the two arguments.
        let args: Vec<TransactionArgument> = vec![
            TransactionArgument::U64(transfer_amount),
            TransactionArgument::Address(*receiver.address()),
        ];

        let txn = sender
            .account()
            .transaction()
            .script(Script::new(
                p2p_script.clone(),
                vec![account_config::xus_tag()],
                args,
            ))
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
            .script(Script::new(
                p2p_script,
                vec![account_config::xus_tag()],
                vec![],
            ))
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
    }
}

#[test]
pub fn test_allowlist() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    executor.set_golden_file(current_function_name!());
    // create an empty transaction
    let sender = executor.create_raw_account_data(1_000_000, 10);
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
    executor.set_golden_file(current_function_name!());

    // create an empty transaction
    let sender = executor.create_raw_account_data(1_000_000, 10);
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
pub fn test_publish_from_diem_root() {
    // create a FakeExecutor with a genesis from file
    let mut executor =
        FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let module = format!(
        "
        module 0x{}.M {{
            public max(a: u64, b: u64): u64 {{
                if (copy(a) > copy(b)) {{
                    return copy(a);
                }} else {{
                    return copy(b);
                }}
                return 0;
            }}

            public sum(a: u64, b: u64): u64 {{
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }}
        }}
        ",
        sender.address(),
    );

    let random_module = compile_module("file_name", &module).1;
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
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, 0);
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
            account_config::XUS_NAME.to_owned(),
            None, /* max_gas_amount */
        );
        assert_prologue_parity!(
            executor.verify_transaction(txn.clone()).status(),
            executor.execute_transaction(txn).status(),
            StatusCode::TRANSACTION_EXPIRED
        );

        // 10 is picked to make sure that SEQUENCE_NUMBER_TOO_NEW will not override the
        // TRANSACTION_EXPIRED error.
        let txn = transaction_test_helpers::get_test_signed_transaction(
            *sender.address(),
            10, /* sequence_number */
            private_key,
            private_key.public_key(),
            None, /* script */
            0,    /* expiration_time */
            0,    /* gas_unit_price */
            account_config::XUS_NAME.to_owned(),
            None, /* max_gas_amount */
        );
        assert_prologue_parity!(
            executor.verify_transaction(txn.clone()).status(),
            executor.execute_transaction(txn).status(),
            StatusCode::TRANSACTION_EXPIRED
        );
    }
    }
}

#[test]
fn verify_chain_id() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, 0);
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
    }
}

#[test]
fn verify_gas_currency_with_bad_identifier() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, 0);
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
            // The gas currency code must be composed of alphanumeric characters and the
            // first character must be a letter.
            "Bad_ID".to_string(),
            None, /* max_gas_amount */
        );
        assert_prologue_parity!(
            executor.verify_transaction(txn.clone()).status(),
            executor.execute_transaction(txn).status(),
            StatusCode::INVALID_GAS_SPECIFIER
        );
    }
    }
}

#[test]
fn verify_gas_currency_code() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, 0);
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
    }
}

#[test]
fn verify_max_sequence_number() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(900_000, std::u64::MAX);
        executor.add_account_data(&sender);
        let private_key = &sender.account().privkey;
        let txn = transaction_test_helpers::get_test_signed_transaction(
            *sender.address(),
            std::u64::MAX, /* sequence_number */
            private_key,
            private_key.public_key(),
            None,     /* script */
            u64::MAX, /* expiration_time */
            0,        /* gas_unit_price */
            "XUS".to_string(),
            None, /* max_gas_amount */
        );
        assert_prologue_parity!(
            executor.verify_transaction(txn.clone()).status(),
            executor.execute_transaction(txn).status(),
            StatusCode::SEQUENCE_NUMBER_TOO_BIG
        );
    }
    }
}

#[test]
pub fn test_no_publishing_diem_root_sender() {
    // create a FakeExecutor with a genesis from file
    let mut executor =
        FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = Account::new_diem_root();

    let module = String::from(
        "
        module 0x1.M {
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

    let random_module = compile_module("file_name", &module).1;
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
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    let receiver = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let module = format!(
        "
        module 0x{}.M {{
            public max(a: u64, b: u64): u64 {{
                if (copy(a) > copy(b)) {{
                    return copy(a);
                }} else {{
                    return copy(b);
                }}
                return 0;
            }}

            public sum(a: u64, b: u64): u64 {{
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }}
        }}
        ",
        receiver.address(),
    );

    let random_module = compile_module("file_name", &module).1;
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
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = format!(
        "
        module 0x{}.M {{
            public max(a: u64, b: u64): u64 {{
                if (copy(a) > copy(b)) {{
                    return copy(a);
                }} else {{
                    return copy(b);
                }}
                return 0;
            }}

            public sum(a: u64, b: u64): u64 {{
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }}
        }}
        ",
        sender.address(),
    );

    let random_module = compile_module("file_name", &program).1;
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

fn bad_module() -> (CompiledModule, Vec<u8>) {
    let bad_module_code = "
    module 0x1.Test {
        struct R1 { b: bool }
        struct S1 has copy, drop { r1: Self.R1 }

        public new_S1(): Self.S1 {
            let s: Self.S1;
            let r: Self.R1;
            r = R1 { b: true };
            s = S1 { r1: move(r) };
            return move(s);
        }
    }
    ";
    let compiler = Compiler { deps: vec![] };
    let module = compiler
        .into_compiled_module("file_name", bad_module_code)
        .expect("Failed to compile");
    let mut bytes = vec![];
    module.serialize(&mut bytes).unwrap();
    (module, bytes)
}

fn good_module_uses_bad(
    address: AccountAddress,
    bad_dep: CompiledModule,
) -> (CompiledModule, Vec<u8>) {
    let good_module_code = format!(
        "
    module 0x{}.Test2 {{
        import 0x1.Test;
        struct S {{ b: bool }}

        foo(): Test.S1 {{
            return Test.new_S1();
        }}
        public bar() {{
            return;
        }}
    }}
    ",
        address,
    );

    let compiler = Compiler {
        deps: diem_framework_releases::current_modules()
            .iter()
            .chain(std::iter::once(&bad_dep))
            .collect(),
    };
    let module = compiler
        .into_compiled_module("file_name", good_module_code.as_str())
        .expect("Failed to compile");
    let mut bytes = vec![];
    module.serialize(&mut bytes).unwrap();
    (module, bytes)
}

#[test]
fn test_script_dependency_fails_verification() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());
    executor.set_golden_file(current_function_name!());

    // Get a module that fails verification into the store.
    let (module, bytes) = bad_module();
    executor.add_module(&module.self_id(), bytes);

    // Create a module that tries to use that module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
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
        deps: vec![&module],
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
    executor.set_golden_file(current_function_name!());

    // Get a module that fails verification into the store.
    let (bad_module, bad_module_bytes) = bad_module();
    executor.add_module(&bad_module.self_id(), bad_module_bytes);

    // Create a transaction that tries to use that module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);
    let good_module = {
        let (_, serialized_module) = good_module_uses_bad(*sender.address(), bad_module);
        diem_types::transaction::Module::new(serialized_module)
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
    executor.set_golden_file(current_function_name!());

    // Get a module that fails verification into the store.
    let (module, bytes) = bad_module();
    executor.add_module(&module.self_id(), bytes);

    // Create a transaction that tries to use that module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    main<T>() {
        return;
    }
    ";

    let compiler = Compiler {
        deps: vec![&module],
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
    executor.set_golden_file(current_function_name!());

    // Get a module that fails verification into the store.
    let (bad_module, bad_module_bytes) = bad_module();
    executor.add_module(&bad_module.self_id(), bad_module_bytes);

    // Create a module that tries to use that module.
    let (good_module, good_module_bytes) =
        good_module_uses_bad(account_config::CORE_CODE_ADDRESS, bad_module);
    executor.add_module(&good_module.self_id(), good_module_bytes);

    // Create a transaction that tries to use that module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    import 0x1.Test2;

    main() {
        Test2.bar();
        return;
    }
    ";

    let compiler = Compiler {
        deps: vec![&good_module],
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
    executor.set_golden_file(current_function_name!());

    // Get a module that fails verification into the store.
    let (bad_module, bad_module_bytes) = bad_module();
    executor.add_module(&bad_module.self_id(), bad_module_bytes);

    // Create a module that tries to use that module.
    let (good_module, good_module_bytes) =
        good_module_uses_bad(account_config::CORE_CODE_ADDRESS, bad_module);
    executor.add_module(&good_module.self_id(), good_module_bytes);

    // Create a transaction that tries to use that module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let module_code = format!(
        "
    module 0x{}.Test3 {{
        import 0x1.Test2;
        public bar() {{
            Test2.bar();
            return;
        }}
    }}
    ",
        sender.address()
    );
    let module = {
        let compiler = Compiler {
            deps: vec![&good_module],
        };
        diem_types::transaction::Module::new(
            compiler
                .into_module_blob("file_name", module_code.as_str())
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
    executor.set_golden_file(current_function_name!());

    // Get a module that fails verification into the store.
    let (bad_module, bad_module_bytes) = bad_module();
    executor.add_module(&bad_module.self_id(), bad_module_bytes);

    // Create a module that tries to use that module.
    let (good_module, good_module_bytes) =
        good_module_uses_bad(account_config::CORE_CODE_ADDRESS, bad_module);
    executor.add_module(&good_module.self_id(), good_module_bytes);

    // Create a transaction that tries to use that module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let code = "
    main<T>() {
        return;
    }
    ";

    let compiler = Compiler {
        deps: vec![&good_module],
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
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let sender = executor.create_raw_account_data(1_000_000, 0);
        executor.add_account_data(&sender);

        // get a SignedTransaction
        let txn = sender
            .account()
            .transaction()
            .script(Script::new(
                LegacyStdlibScript::PeerToPeerWithMetadata
                    .compiled_bytes()
                    .into_vec(),
                vec![account_config::xus_tag()],
                // Don't pass any arguments
                vec![],
            ))
            .sequence_number(0)
            .max_gas_amount(gas_costs::TXN_RESERVED)
            .sign();

        let output = executor.execute_transaction(txn);
        assert!(!output.status().is_discarded());
        assert!(output.gas_used() > 0);
    }
    }
}

#[test]
pub fn publish_and_register_new_currency() {
    // Test creating and registering a new currency and verify that it can
    // only be used to pay transaction fees after it is initialized for that
    // purpose.

    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = Account::new_diem_root();
    let tc_account = Account::new_blessed_tc();

    let module = r#"
        module 0x1.COIN {
            import 0x1.FixedPoint32;
            import 0x1.Diem;
            struct COIN has store { x: bool }
            public initialize(dr_account: &signer, tc_account: &signer) {
                Diem.register_SCS_currency<Self.COIN>(
                    move(dr_account),
                    move(tc_account),
                    FixedPoint32.create_from_rational(1,2),
                    100000,
                    100,
                    h"434f494e",
                );
                return;
            }
        }
    "#;

    let (compiled_module, module) = compile_module("file_name", module);
    let txn = sender
        .transaction()
        .module(module)
        .sequence_number(1)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_and_apply(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    let coin_tag = account_config::type_tag_for_currency_code(Identifier::new("COIN").unwrap());

    {
        let program = {
            let code = r#"
            import 0x1.COIN;
            main(lr_account: signer, tc_account: signer) {
                COIN.initialize(&lr_account, &tc_account);
                return;
            }
            "#;
            let compiler = Compiler {
                deps: vec![&compiled_module],
            };
            compiler
                .into_script_blob("file_name", code)
                .expect("Failed to compile")
        };
        let txn = sender
            .transaction()
            .write_set(WriteSetPayload::Script {
                script: Script::new(program, vec![], vec![]),
                execute_as: *tc_account.address(),
            })
            .sequence_number(2)
            .sign();
        executor.new_block();
        executor.execute_and_apply(txn);
    }

    let dd = Account::new_from_seed(&mut KeyGen::from_seed([0; 32]));

    let txn = tc_account
        .transaction()
        .script(transaction_builder::encode_create_designated_dealer_script(
            coin_tag.clone(),
            0,
            *dd.address(),
            dd.auth_key_prefix(),
            b"".to_vec(),
            true,
        ))
        .sequence_number(0)
        .sign();

    executor.execute_and_apply(txn);

    executor.exec(
        "DesignatedDealer",
        "add_currency",
        vec![coin_tag.clone()],
        serialize_values(&vec![
            MoveValue::Signer(*dd.address()),
            MoveValue::Signer(*tc_account.address()),
        ]),
    );

    let txn = tc_account
        .transaction()
        .script(transaction_builder::encode_tiered_mint_script(
            coin_tag.clone(),
            0,
            *dd.address(),
            50000,
            1,
        ))
        .sequence_number(1)
        .sign();

    executor.execute_and_apply(txn);

    let txn = dd
        .transaction()
        .script(
            transaction_builder::encode_peer_to_peer_with_metadata_script(
                coin_tag.clone(),
                *dd.address(),
                1,
                b"".to_vec(),
                b"".to_vec(),
            ),
        )
        .gas_unit_price(1)
        .max_gas_amount(800)
        .gas_currency_code("COIN")
        .sequence_number(0)
        .sign();

    let balance = executor.read_balance_resource(&dd, Identifier::new("COIN").unwrap());
    assert!(balance.unwrap().coin() > 800);

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn.clone()).status(),
        StatusCode::BAD_TRANSACTION_FEE_CURRENCY
    );

    executor.exec(
        "TransactionFee",
        "add_txn_fee_currency",
        vec![coin_tag],
        serialize_values(&vec![MoveValue::Signer(*tc_account.address())]),
    );

    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}
