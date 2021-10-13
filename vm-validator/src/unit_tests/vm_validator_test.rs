// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::{TransactionValidation, VMValidator};
use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_transaction_builder::stdlib::encode_peer_to_peer_with_metadata_script;
use diem_types::{
    account_address, account_config,
    account_config::{xus_tag, XUS_NAME},
    chain_id::ChainId,
    test_helpers::transaction_test_helpers,
    transaction::{Module, Script, TransactionArgument, TransactionPayload},
    vm_status::StatusCode,
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use move_core_types::gas_schedule::{GasAlgebra, GasConstants, MAX_TRANSACTION_SIZE_IN_BYTES};
use rand::SeedableRng;
use std::u64;
use storage_interface::DbReaderWriter;

struct TestValidator {
    vm_validator: VMValidator,
    _db_path: diem_temppath::TempPath,
}

impl TestValidator {
    fn new() -> Self {
        let _db_path = diem_temppath::TempPath::new();
        _db_path.create_as_dir().unwrap();
        let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(_db_path.path()));
        executor_test_helpers::bootstrap_genesis::<DiemVM>(
            &db_rw,
            &vm_genesis::test_genesis_transaction(),
        )
        .expect("Db-bootstrapper should not fail.");

        // Create another client for the vm_validator since the one used for the executor will be
        // run on another runtime which will be dropped before this function returns.
        let vm_validator = VMValidator::new(db);
        TestValidator {
            vm_validator,
            _db_path,
        }
    }
}

impl std::ops::Deref for TestValidator {
    type Target = VMValidator;

    fn deref(&self) -> &Self::Target {
        &self.vm_validator
    }
}

// These tests are meant to test all high-level code paths that lead to a validation error in the
// verification of a transaction in the VM. However, there are a couple notable exceptions that we
// do _not_ test here -- this is due to limitations around execution and semantics. The following
// errors are not exercised:
// * SEQUENCE_NUMBER_TOO_OLD -- We can't test sequence number too old here without running execution
//   first in order to bump the account's sequence number. This needs to (and is) tested in the
//   language e2e tests in: diem/language/e2e-testsuite/src/tests/verify_txn.rs ->
//   verify_simple_payment.
// * SEQUENCE_NUMBER_TOO_NEW -- This error is filtered out when running validation; it is only
//   testable when running the executor.
// * INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE -- This is tested in verify_txn.rs.
// * SENDING_ACCOUNT_FROZEN: Tested in functional-tests/tests/diem_account/freezing.move.
// * Errors arising from deserializing the code -- these are tested in
//   - diem/language/move-binary-format/src/unit_tests/deserializer_tests.rs
//   - diem/language/move-binary-format/tests/serializer_tests.rs
// * Errors arising from calls to `static_verify_program` -- this is tested separately in tests for
//   the bytecode verifier.
// * Testing for invalid genesis write sets -- this is tested in
//   diem/language/e2e-testsuite/src/tests/genesis.rs

#[test]
fn test_validate_transaction() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let program = encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(program)),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
}

#[test]
fn test_validate_invalid_signature() {
    let vm_validator = TestValidator::new();

    let mut rng = ::rand::rngs::StdRng::from_seed([1u8; 32]);
    let other_private_key = Ed25519PrivateKey::generate(&mut rng);
    // Submit with an account using an different private/public keypair

    let address = account_config::diem_root_address();
    let program = encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_unchecked_txn(
        address,
        1,
        &other_private_key,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(program),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_SIGNATURE);
}

#[test]
fn test_validate_known_script_too_large_args() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(Script::new(
            vec![42; MAX_TRANSACTION_SIZE_IN_BYTES as usize],
            vec![],
            vec![],
        ))),
        /* generate a
         * program with args
         * longer than the
         * max size */
        0,
        0,                   /* max gas price */
        XUS_NAME.to_owned(), /* gas currency code */
        None,
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE
    );
}

#[test]
fn test_validate_max_gas_units_above_max() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None,
        0,
        0,                   /* max gas price */
        XUS_NAME.to_owned(), /* gas currency code */
        Some(u64::MAX),      // Max gas units
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND
    );
}

#[test]
fn test_validate_max_gas_units_below_min() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    // Calculate a size for the transaction script that will ensure
    // that the minimum transaction gas is at least 1 after scaling to the
    // external gas units.
    let gas_constants = &GasConstants::default();
    let txn_bytes = gas_constants.large_transaction_cutoff.get()
        + (gas_constants.gas_unit_scaling_factor / gas_constants.intrinsic_gas_per_byte.get());
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(Script::new(
            vec![42; txn_bytes as usize],
            vec![],
            vec![],
        ))),
        0,
        0,                   /* max gas price */
        XUS_NAME.to_owned(), /* gas currency code */
        Some(0),             // Max gas units
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS
    );
}

#[test]
fn test_validate_max_gas_price_above_bounds() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None,
        0,
        u64::MAX,            /* max gas price */
        XUS_NAME.to_owned(), /* gas currency code */
        None,
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND
    );
}

// NB: This test is designed to fail if/when we bump the minimum gas price to be non-zero. You will
// then need to update this price here in order to make the test pass -- uncomment the commented
// out assertion and remove the current failing assertion in this case.
#[test]
fn test_validate_max_gas_price_below_bounds() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let program = encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(program)),
        // Initial Time was set to 0 with a TTL 86400 secs.
        40000,
        0,                   /* max gas price */
        XUS_NAME.to_owned(), /* gas currency code */
        None,
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
    //assert_eq!(
    //    ret.status().unwrap().major_status,
    //    StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND
    //);
}

#[test]
fn test_validate_unknown_script() {
    let vm_validator = TestValidator::new();

    let address = account_config::testnet_dd_account_address();
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(Script::new(
            vec![],
            vec![],
            vec![],
        ))),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    println!("{:?}", ret);
    assert_eq!(ret.status().unwrap(), StatusCode::UNKNOWN_SCRIPT);
}

// Make sure that we can publish non-allowlisted modules from the association address
#[test]
fn test_validate_module_publishing() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_module_publishing_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Module::new(vec![]),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
}

// Make sure that we can't publish non-allowlisted modules
#[test]
fn test_validate_module_publishing_non_association() {
    let vm_validator = TestValidator::new();

    let address = account_config::treasury_compliance_account_address();
    let transaction = transaction_test_helpers::get_test_signed_module_publishing_transaction(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Module::new(vec![]),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_MODULE_PUBLISHER);
}

#[test]
fn test_validate_invalid_auth_key() {
    let vm_validator = TestValidator::new();

    let mut rng = ::rand::rngs::StdRng::from_seed([1u8; 32]);
    let other_private_key = Ed25519PrivateKey::generate(&mut rng);
    // Submit with an account using an different private/public keypair

    let address = account_config::diem_root_address();
    let program = encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &other_private_key,
        other_private_key.public_key(),
        Some(TransactionPayload::Script(program)),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_AUTH_KEY);
}

#[test]
fn test_validate_account_doesnt_exist() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let random_account_addr = account_address::AccountAddress::random();
    let program = encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        random_account_addr,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(program)),
        0,
        1,                   /* max gas price */
        XUS_NAME.to_owned(), /* gas currency code */
        None,
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
    );
}

#[test]
fn test_validate_sequence_number_too_new() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let program = encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(program)),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
}

#[test]
fn test_validate_invalid_arguments() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let (program_script, _, _) =
        encode_peer_to_peer_with_metadata_script(xus_tag(), address, 100, vec![], vec![])
            .into_inner();
    let program = Script::new(program_script, vec![], vec![TransactionArgument::U64(42)]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        Some(TransactionPayload::Script(program)),
    );
    let _ret = vm_validator.validate_transaction(transaction).unwrap();
    // TODO: Script arguement types are now checked at execution time. Is this an idea behavior?
    // assert_eq!(ret.status().unwrap().major_status, StatusCode::TYPE_MISMATCH);
}

#[test]
fn test_validate_non_genesis_write_set() {
    let vm_validator = TestValidator::new();

    // Confirm that a correct transaction is validated successfully.
    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_write_set_txn(
        address,
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None,
    )
    .into_inner();
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert!(ret.status().is_none());

    // A WriteSet txn is only valid when sent from the Diem root account.
    let bad_transaction = transaction_test_helpers::get_write_set_txn(
        account_config::treasury_compliance_account_address(),
        1,
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None,
    )
    .into_inner();
    let ret = vm_validator.validate_transaction(bad_transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::REJECTED_WRITE_SET);
}

#[test]
fn test_validate_expiration_time() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1, /* sequence_number */
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None, /* script */
        0,    /* expiration_time */
        0,    /* gas_unit_price */
        XUS_NAME.to_owned(),
        None, /* max_gas_amount */
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::TRANSACTION_EXPIRED);
}

#[test]
fn test_validate_chain_id() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_txn_with_chain_id(
        address,
        0, /* sequence_number */
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        // all tests use ChainId::test() for chain_id, so pick something different
        ChainId::new(ChainId::test().id() + 1),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::BAD_CHAIN_ID);
}

#[test]
fn test_validate_gas_currency_with_bad_identifier() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1, /* sequence_number */
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None,     /* script */
        u64::MAX, /* expiration_time */
        0,        /* gas_unit_price */
        // The gas currency code must be composed of alphanumeric characters and the
        // first character must be a letter.
        "Bad_ID".to_string(),
        None, /* max_gas_amount */
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_GAS_SPECIFIER);
}

#[test]
fn test_validate_gas_currency_code() {
    let vm_validator = TestValidator::new();

    let address = account_config::diem_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1, /* sequence_number */
        &vm_genesis::GENESIS_KEYPAIR.0,
        vm_genesis::GENESIS_KEYPAIR.1.clone(),
        None,     /* script */
        u64::MAX, /* expiration_time */
        0,        /* gas_unit_price */
        "INVALID".to_string(),
        None, /* max_gas_amount */
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::CURRENCY_INFO_DOES_NOT_EXIST
    );
}
