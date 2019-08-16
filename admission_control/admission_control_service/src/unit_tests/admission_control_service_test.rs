// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    admission_control_service::{
        AdmissionControlService, SubmitTransactionRequest,
        SubmitTransactionResponse as ProtoSubmitTransactionResponse,
    },
    unit_tests::LocalMockMempool,
};
use admission_control_proto::{AdmissionControlStatus, SubmitTransactionResponse};

use crypto::{ed25519::*, test_utils::TEST_SEED};
use mempool::proto::shared::mempool_status::MempoolAddTransactionStatusCode;
use proto_conv::FromProto;
use rand::SeedableRng;
use std::sync::Arc;
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

fn create_ac_service_for_ut() -> AdmissionControlService<LocalMockMempool, MockVMValidator> {
    AdmissionControlService::new(
        Some(Arc::new(LocalMockMempool::new())),
        Arc::new(MockStorageReadClient),
        Arc::new(MockVMValidator),
        false,
    )
}

fn assert_status(response: ProtoSubmitTransactionResponse, status: VMStatus) {
    let rust_resp = SubmitTransactionResponse::from_proto(response).unwrap();
    if rust_resp.ac_status.is_some() {
        assert_eq!(
            rust_resp.ac_status.unwrap(),
            AdmissionControlStatus::Accepted
        );
    } else {
        let decoded_response = rust_resp.vm_error.unwrap();
        assert_eq!(decoded_response, status)
    }
}

#[test]
fn test_submit_txn_inner_vm() {
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let ac_service = create_ac_service_for_ut();
    // create request
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::new();
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::SendingAccountDoesNotExist(
            "TEST".to_string(),
        )),
    );
    let sender = AccountAddress::new([1; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InvalidSignature),
    );
    let sender = AccountAddress::new([2; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InsufficientBalanceForTransactionFee),
    );
    let sender = AccountAddress::new([3; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::SequenceNumberTooNew),
    );
    let sender = AccountAddress::new([4; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::SequenceNumberTooOld),
    );
    let sender = AccountAddress::new([5; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::TransactionExpired),
    );
    let sender = AccountAddress::new([6; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InvalidAuthKey),
    );
    let sender = AccountAddress::new([8; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::Execution(ExecutionStatus::Executed));

    let sender = AccountAddress::new([8; ADDRESS_LENGTH]);
    let test_key = compat::generate_keypair(&mut rng);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        test_key.1.clone(),
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InvalidSignature),
    );
}

#[test]
fn test_submit_txn_inner_mempool() {
    let ac_service = create_ac_service_for_ut();
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::new();
    let keypair = compat::generate_keypair(None);
    let insufficient_balance_add = AccountAddress::new([100; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        insufficient_balance_add,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::InsufficientBalance
    );
    let invalid_seq_add = AccountAddress::new([101; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        invalid_seq_add,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::InvalidSeqNumber
    );
    let sys_error_add = AccountAddress::new([102; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sys_error_add,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::InvalidUpdate
    );
    let accepted_add = AccountAddress::new([103; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        accepted_add,
        0,
        keypair.0.clone(),
        keypair.1.clone(),
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.ac_status.unwrap(),
        AdmissionControlStatus::Accepted,
    );
}
