// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::{TransactionValidation, VMValidator};
use config_builder::util::get_test_config;
use executor::Executor;
use futures::future::Future;
use grpc_helpers::ServerHandle;
use grpcio::EnvBuilder;
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::*;
use libra_types::{
    account_address, account_config,
    test_helpers::transaction_test_helpers,
    transaction::{Module, Script, TransactionArgument, MAX_TRANSACTION_SIZE_IN_BYTES},
    vm_error::StatusCode,
};
use rand::SeedableRng;
use std::{sync::Arc, u64};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use transaction_builder::encode_transfer_script;
use vm_runtime::MoveVM;

struct TestValidator {
    _storage: ServerHandle,
    vm_validator: VMValidator,
}

impl TestValidator {
    fn new(config: &NodeConfig) -> Self {
        let storage = start_storage_service(&config);

        // setup execution
        let client_env = Arc::new(EnvBuilder::new().build());
        let storage_read_client: Arc<dyn StorageRead> = Arc::new(StorageReadServiceClient::new(
            Arc::clone(&client_env),
            &config.storage.address,
            config.storage.port,
        ));
        let storage_write_client = Arc::new(StorageWriteServiceClient::new(
            Arc::clone(&client_env),
            &config.storage.address,
            config.storage.port,
            None,
        ));

        // Create executor to initialize genesis state. Otherwise gprc will report error when
        // fetching data from storage.
        let _executor = Executor::<MoveVM>::new(
            Arc::clone(&storage_read_client) as Arc<dyn StorageRead>,
            storage_write_client,
            config,
        );

        let vm_validator = VMValidator::new(config, storage_read_client);

        TestValidator {
            _storage: storage,
            vm_validator,
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
// * Sequence number too old -- We can't test sequence number too old here without running execution
//   first in order to bump the account's sequence number. This needs to (and is) tested in the
//   language e2e tests in: libra/language/e2e-tests/src/tests/verify_txn.rs ->
//   verify_simple_payment.
// * Errors arising from deserializing the code -- these are tested in
//   - libra/language/vm/src/unit_tests/deserializer_tests.rs
//   - libra/language/vm/tests/serializer_tests.rs
// * Errors arising from calls to `static_verify_program` -- this is tested separately in tests for
//   the bytecode verifier.
// * Testing for invalid genesis write sets -- this is tested in
//   libra/language/e2e-tests/src/tests/genesis.rs

#[test]
fn test_validate_transaction() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_script(&address, 100);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret, None);
}

#[test]
fn test_validate_invalid_signature() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let mut rng = ::rand::rngs::StdRng::from_seed([1u8; 32]);
    let (other_private_key, _) = compat::generate_keypair(&mut rng);
    // Submit with an account using an different private/public keypair

    let address = account_config::association_address();
    let program = encode_transfer_script(&address, 100);
    let transaction = transaction_test_helpers::get_test_unchecked_txn(
        address,
        1,
        other_private_key,
        keypair.public_key,
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret.unwrap().major_status, StatusCode::INVALID_SIGNATURE);
}

#[test]
fn test_validate_known_script_too_large_args() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        Some(Script::new(vec![42; MAX_TRANSACTION_SIZE_IN_BYTES], vec![])), /* generate a
                                                                             * program with args
                                                                             * longer than the
                                                                             * max size */
        0,
        0, /* max gas price */
        None,
    );
    let ret = vm_validator.validate_transaction(txn).wait().unwrap();
    assert_eq!(
        ret.unwrap().major_status,
        StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE
    );
}

#[test]
fn test_validate_max_gas_units_above_max() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        None,
        0,
        0,              /* max gas price */
        Some(u64::MAX), // Max gas units
    );
    let ret = vm_validator.validate_transaction(txn).wait().unwrap();
    assert_eq!(
        ret.unwrap().major_status,
        StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND
    );
}

#[test]
fn test_validate_max_gas_units_below_min() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        None,
        0,
        0,       /* max gas price */
        Some(1), // Max gas units
    );
    let ret = vm_validator.validate_transaction(txn).wait().unwrap();
    assert_eq!(
        ret.unwrap().major_status,
        StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS
    );
}

#[test]
fn test_validate_max_gas_price_above_bounds() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        None,
        0,
        u64::MAX, /* max gas price */
        None,
    );
    let ret = vm_validator.validate_transaction(txn).wait().unwrap();
    assert_eq!(
        ret.unwrap().major_status,
        StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND
    );
}

// NB: This test is designed to fail if/when we bump the minimum gas price to be non-zero. You will
// then need to update this price here in order to make the test pass -- uncomment the commented
// out assertion and remove the current failing assertion in this case.
#[test]
fn test_validate_max_gas_price_below_bounds() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_script(&address, 100);
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        Some(program),
        0,
        0, /* max gas price */
        None,
    );
    let ret = vm_validator.validate_transaction(txn).wait().unwrap();
    assert_eq!(ret, None);
    //assert_eq!(
    //    ret.unwrap().major_status,
    //    StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND
    //);
}

#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[test]
fn test_validate_unknown_script() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        None,
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret.unwrap().major_status, StatusCode::UNKNOWN_SCRIPT);
}

// Make sure that we can't publish non-whitelisted modules
#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[cfg(not(feature = "custom_modules"))]
#[test]
fn test_validate_module_publishing() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let transaction = transaction_test_helpers::get_test_signed_module_publishing_transaction(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        Module::new(vec![]),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret.unwrap().major_status, StatusCode::UNKNOWN_MODULE);
}

#[test]
fn test_validate_invalid_auth_key() {
    let (config, _) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let mut rng = ::rand::rngs::StdRng::from_seed([1u8; 32]);
    let (other_private_key, other_public_key) = compat::generate_keypair(&mut rng);
    // Submit with an account using an different private/public keypair

    let address = account_config::association_address();
    let program = encode_transfer_script(&address, 100);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        other_private_key,
        other_public_key,
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret.unwrap().major_status, StatusCode::INVALID_AUTH_KEY);
}

#[test]
fn test_validate_balance_below_gas_fee() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_script(&address, 100);
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        address,
        1,
        keypair.private_key.clone(),
        keypair.public_key,
        Some(program),
        0,
        // Note that this will be dependent upon the max gas price and gas amounts that are set. So
        // changing those may cause this test to fail.
        10_000, /* max gas price */
        Some(1_000_000),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(
        ret.unwrap().major_status,
        StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE
    );
}

#[test]
fn test_validate_account_doesnt_exist() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let random_account_addr = account_address::AccountAddress::random();
    let program = encode_transfer_script(&address, 100);
    let transaction = transaction_test_helpers::get_test_signed_transaction(
        random_account_addr,
        1,
        keypair.private_key,
        keypair.public_key,
        Some(program),
        0,
        1, /* max gas price */
        None,
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(
        ret.unwrap().major_status,
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
    );
}

#[test]
fn test_validate_sequence_number_too_new() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_script(&address, 100);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret, None);
}

#[test]
fn test_validate_invalid_arguments() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let (program_script, _) = encode_transfer_script(&address, 100).into_inner();
    let program = Script::new(program_script, vec![TransactionArgument::U64(42)]);
    let transaction = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret.unwrap().major_status, StatusCode::TYPE_MISMATCH);
}

#[test]
fn test_validate_non_genesis_write_set() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let transaction = transaction_test_helpers::get_write_set_txn(
        address,
        1,
        keypair.private_key,
        keypair.public_key,
        None,
    )
    .into_inner();
    let ret = vm_validator
        .validate_transaction(transaction)
        .wait()
        .unwrap();
    assert_eq!(ret.unwrap().major_status, StatusCode::REJECTED_WRITE_SET);
}
