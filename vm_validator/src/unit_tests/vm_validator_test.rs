// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::{TransactionValidation, VMValidator};
use assert_matches::assert_matches;
use config::config::NodeConfig;
use config_builder::util::get_test_config;
use crypto::signing::KeyPair;
use execution_proto::proto::execution_grpc;
use execution_service::ExecutionService;
use futures::future::Future;
use grpc_helpers::ServerHandle;
use grpcio::EnvBuilder;
use proto_conv::FromProto;
use std::{sync::Arc, u64};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use types::{
    account_address, account_config,
    test_helpers::transaction_test_helpers,
    transaction::{Program, SignedTransaction, TransactionArgument, MAX_TRANSACTION_SIZE_IN_BYTES},
    vm_error::{VMStatus, VMValidationStatus, VMVerificationError, VMVerificationStatus},
};
use vm_genesis::encode_transfer_program;

struct TestValidator {
    _storage: ServerHandle,
    _execution: grpcio::Server,
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
        ));

        let handle = ExecutionService::new(
            Arc::clone(&storage_read_client),
            storage_write_client,
            config,
        );
        let service = execution_grpc::create_execution(handle);
        let execution = ::grpcio::ServerBuilder::new(Arc::new(EnvBuilder::new().build()))
            .register_service(service)
            .bind(config.execution.address.clone(), config.execution.port)
            .build()
            .expect("Unable to create grpc server");

        let vm_validator = VMValidator::new(config, storage_read_client);

        TestValidator {
            _storage: storage,
            _execution: execution,
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
//   vm_runtime_tests in: libra/language/vm/vm_runtime/vm_runtime_tests/src/tests/verify_txn.rs ->
//   verify_simple_payment.
// * Errors arising from deserializing the code -- these are tested in
//   - libra/language/vm/src/unit_tests/deserializer_tests.rs
//   - libra/language/vm/tests/serializer_tests.rs
// * Errors arising from calls to `static_verify_program` -- this is tested separately in tests for
//   the bytecode verifier.
// * Testing for invalid genesis write sets -- this is tested in
//   libra/language/vm/vm_runtime/vm_runtime_tests/src/tests/genesis.rs

#[test]
fn test_validate_transaction() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_program(&address, 100);
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(ret, None);
}

#[test]
fn test_validate_invalid_signature() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let (other_private_key, _) = ::crypto::signing::generate_keypair();
    // Submit with an account wusing an different private/public keypair
    let other_keypair = KeyPair::new(other_private_key);

    let address = account_config::association_address();
    let program = encode_transfer_program(&address, 100);
    let signed_txn = transaction_test_helpers::get_unverified_test_signed_txn(
        address,
        0,
        other_keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(signed_txn)
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::InvalidSignature))
    );
}

#[test]
fn test_validate_known_script_too_large_args() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(Program::new(
            vec![42; MAX_TRANSACTION_SIZE_IN_BYTES],
            vec![],
            vec![],
        )), /* generate a program with args longer than the max size */
        0,
        0, /* max gas price */
        None,
    );
    let txn = SignedTransaction::from_proto(txn).unwrap();
    let ret = vm_validator.validate_transaction(txn).wait().unwrap();
    assert_matches!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::ExceededMaxTransactionSize(_)))
    );
}

#[test]
fn test_validate_max_gas_units_above_max() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        None,
        0,
        0,              /* max gas price */
        Some(u64::MAX), // Max gas units
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(txn).unwrap())
        .wait()
        .unwrap();
    assert_matches!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::MaxGasUnitsExceedsMaxGasUnitsBound(_)))
    );
}

#[test]
fn test_validate_max_gas_units_below_min() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        None,
        0,
        0,       /* max gas price */
        Some(1), // Max gas units
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(txn).unwrap())
        .wait()
        .unwrap();
    assert_matches!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(_)))
    );
}

#[test]
fn test_validate_max_gas_price_above_bounds() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        None,
        0,
        u64::MAX, /* max gas price */
        None,
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(txn).unwrap())
        .wait()
        .unwrap();
    assert_matches!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::GasUnitPriceAboveMaxBound(_)))
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
    let program = encode_transfer_program(&address, 100);
    let txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
        0,
        0, /* max gas price */
        None,
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(ret, None);
    //assert_eq!(
    //    ret.unwrap(),
    //    VMStatus::ValidationStatus(VMValidationStatus::GasUnitPriceBelowMinBound)
    //);
}

#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[test]
fn test_validate_unknown_script() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key().clone(),
        keypair.public_key(),
        None,
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::UnknownScript))
    );
}

// Make sure that we can't publish non-whitelisted modules
#[cfg(not(feature = "allow_custom_transaction_scripts"))]
#[cfg(not(feature = "custom_modules"))]
#[test]
fn test_validate_module_publishing() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let (program_script, args, _) = encode_transfer_program(&address, 100).into_inner();
    let program = Program::new(program_script, vec![vec![]], args);
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::UnknownModule))
    );
}

#[test]
fn test_validate_invalid_auth_key() {
    let (config, _) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let (other_private_key, _) = ::crypto::signing::generate_keypair();
    // Submit with an account wusing an different private/public keypair
    let other_keypair = KeyPair::new(other_private_key);

    let address = account_config::association_address();
    let program = encode_transfer_program(&address, 100);
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        address,
        0,
        other_keypair.private_key().clone(),
        other_keypair.public_key(),
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::InvalidAuthKey))
    );
}

#[test]
fn test_validate_balance_below_gas_fee() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_program(&address, 100);
    let signed_txn = transaction_test_helpers::get_test_signed_transaction(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
        0,
        // Note that this will be dependent upon the max gas price and gas amounts that are set. So
        // changing those may cause this test to fail.
        10_000, /* max gas price */
        Some(1_000_000),
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Validation(
            VMValidationStatus::InsufficientBalanceForTransactionFee
        ))
    );
}

#[test]
fn test_validate_account_doesnt_exist() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let random_account_addr = account_address::AccountAddress::random();
    let program = encode_transfer_program(&address, 100);
    let signed_txn = transaction_test_helpers::get_test_signed_transaction(
        random_account_addr,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
        0,
        1, /* max gas price */
        None,
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_matches!(
        ret.unwrap(),
        VMStatus::Validation(VMValidationStatus::SendingAccountDoesNotExist(_))
    );
}

#[test]
fn test_validate_sequence_number_too_new() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let program = encode_transfer_program(&address, 100);
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        address,
        1,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(ret, None);
}

#[test]
fn test_validate_invalid_arguments() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let (program_script, _, _) = encode_transfer_program(&address, 100).into_inner();
    let program = Program::new(program_script, vec![], vec![TransactionArgument::U64(42)]);
    let signed_txn = transaction_test_helpers::get_test_signed_txn(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        Some(program),
    );
    let ret = vm_validator
        .validate_transaction(SignedTransaction::from_proto(signed_txn).unwrap())
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Verification(vec![VMVerificationStatus::Script(
            VMVerificationError::TypeMismatch("Actual Type Mismatch".to_string())
        )]))
    );
}

#[test]
fn test_validate_non_genesis_write_set() {
    let (config, keypair) = get_test_config();
    let vm_validator = TestValidator::new(&config);

    let address = account_config::association_address();
    let signed_txn = transaction_test_helpers::get_write_set_txn(
        address,
        0,
        keypair.private_key().clone(),
        keypair.public_key(),
        None,
    );
    let ret = vm_validator
        .validate_transaction(signed_txn)
        .wait()
        .unwrap();
    assert_eq!(
        ret,
        Some(VMStatus::Validation(VMValidationStatus::RejectedWriteSet))
    );
}
