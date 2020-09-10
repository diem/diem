// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::{TransactionValidation, VMValidator};
use libra_config::{config::NodeConfig, utils::get_genesis_txn};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::{
    account_address, account_config,
    account_config::{lbr_type_tag, LBR_NAME},
    test_helpers::transaction_test_helpers,
    transaction::{Module, Script, TransactionArgument, MAX_TRANSACTION_SIZE_IN_BYTES},
    vm_status::StatusCode,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::SeedableRng;
use std::u64;
use storage_interface::DbReaderWriter;
use transaction_builder::encode_peer_to_peer_with_metadata_script;

struct TestValidator {
    vm_validator: VMValidator,
}

impl TestValidator {
    fn new(config: &NodeConfig) -> Self {
        let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(&config.storage.dir()));
        executor_test_helpers::bootstrap_genesis::<LibraVM>(
            &db_rw,
            get_genesis_txn(config).unwrap(),
        )
        .expect("Db-bootstrapper should not fail.");

        // Create another client for the vm_validator since the one used for the executor will be
        // run on another runtime which will be dropped before this function returns.
        let vm_validator = VMValidator::new(db);
        TestValidator { vm_validator }
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
// * Sequence number too old -- We can't test sequence number too old here without running execution
//   first in order to bump the account's sequence number. This needs to (and is) tested in the
//   language e2e tests in: libra/language/e2e-testsuite/src/tests/verify_txn.rs ->
//   verify_simple_payment.
// * Errors arising from deserializing the code -- these are tested in
//   - libra/language/vm/src/unit_tests/deserializer_tests.rs
//   - libra/language/vm/tests/serializer_tests.rs
// * Errors arising from calls to `static_verify_program` -- this is tested separately in tests for
//   the bytecode verifier.
// * Testing for invalid genesis write sets -- this is tested in
//   libra/language/e2e-testsuite/src/tests/genesis.rs

#[test]
fn test_validate_transaction() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let program =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &key,
        key.public_key(),
        Some(program),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
}

#[test]
fn test_validate_invalid_signature() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let mut rng = ::rand::rngs::StdRng::from_seed([1u8; 32]);
    let other_private_key = Ed25519PrivateKey::generate(&mut rng);
    // Submit with an account using an different private/public keypair

    let address = account_config::libra_root_address();
    let program =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_unchecked_txn(
        address,
        1,
        &other_private_key,
        key.public_key(),
        Some(program),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_SIGNATURE);
}

#[test]
fn test_validate_known_script_too_large_args() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &key,
        key.public_key(),
        Some(Script::new(
            vec![42; MAX_TRANSACTION_SIZE_IN_BYTES],
            vec![],
            vec![],
        )), /* generate a
             * program with args
             * longer than the
             * max size */
        0,
        0,                   /* max gas price */
        LBR_NAME.to_owned(), /* gas currency code */
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
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &key,
        key.public_key(),
        None,
        0,
        0,                   /* max gas price */
        LBR_NAME.to_owned(), /* gas currency code */
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
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &key,
        key.public_key(),
        None,
        0,
        0,                   /* max gas price */
        LBR_NAME.to_owned(), /* gas currency code */
        Some(1),             // Max gas units
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(
        ret.status().unwrap(),
        StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS
    );
}

#[test]
fn test_validate_max_gas_price_above_bounds() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &key,
        key.public_key(),
        None,
        0,
        u64::MAX,            /* max gas price */
        LBR_NAME.to_owned(), /* gas currency code */
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
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let program =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        &key,
        key.public_key(),
        Some(program),
        // Initial Time was set to 0 with a TTL 86400 secs.
        40000,
        0,                   /* max gas price */
        LBR_NAME.to_owned(), /* gas currency code */
        None,
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
    //assert_eq!(
    //    ret.status().unwrap().major_status,
    //    StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND
    //);
}

#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[test]
fn test_validate_unknown_script() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::testnet_dd_account_address();
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &key,
        key.public_key(),
        Some(Script::new(vec![], vec![], vec![])),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::UNKNOWN_SCRIPT);
}

// Make sure that we can publish non-allowlisted modules from the association address
#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[cfg(not(feature = "custom_modules"))]
#[test]
fn test_validate_module_publishing() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let transaction = transaction_test_helpers::get_test_signed_module_publishing_transaction(
        address,
        1,
        &key,
        key.public_key(),
        Module::new(vec![]),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
}

// Make sure that we can't publish non-allowlisted modules
#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[cfg(not(feature = "custom_modules"))]
#[test]
fn test_validate_module_publishing_non_association() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::treasury_compliance_account_address();
    let transaction = transaction_test_helpers::get_test_signed_module_publishing_transaction(
        address,
        1,
        &key,
        key.public_key(),
        Module::new(vec![]),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_MODULE_PUBLISHER);
}

#[test]
fn test_validate_invalid_auth_key() {
    let (config, _) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let mut rng = ::rand::rngs::StdRng::from_seed([1u8; 32]);
    let other_private_key = Ed25519PrivateKey::generate(&mut rng);
    // Submit with an account using an different private/public keypair

    let address = account_config::libra_root_address();
    let program =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &other_private_key,
        other_private_key.public_key(),
        Some(program),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::INVALID_AUTH_KEY);
}

#[test]
fn test_validate_account_doesnt_exist() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let random_account_addr = account_address::AccountAddress::random();
    let program =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        random_account_addr,
        1,
        &key,
        key.public_key(),
        Some(program),
        0,
        1,                   /* max gas price */
        LBR_NAME.to_owned(), /* gas currency code */
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
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let program =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &key,
        key.public_key(),
        Some(program),
    );
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status(), None);
}

#[test]
fn test_validate_invalid_arguments() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let (program_script, _) =
        encode_peer_to_peer_with_metadata_script(lbr_type_tag(), address, 100, vec![], vec![])
            .into_inner();
    let program = Script::new(program_script, vec![], vec![TransactionArgument::U64(42)]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        &key,
        key.public_key(),
        Some(program),
    );
    let _ret = vm_validator.validate_transaction(transaction).unwrap();
    // TODO: Script arguement types are now checked at execution time. Is this an idea behavior?
    // assert_eq!(ret.status().unwrap().major_status, StatusCode::TYPE_MISMATCH);
}

#[test]
fn test_validate_non_genesis_write_set() {
    let (config, key) = config_builder::test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::libra_root_address();
    let transaction =
        transaction_test_helpers::get_write_set_txn(address, 2, &key, key.public_key(), None)
            .into_inner();
    let ret = vm_validator.validate_transaction(transaction).unwrap();
    assert_eq!(ret.status().unwrap(), StatusCode::REJECTED_WRITE_SET);
}
