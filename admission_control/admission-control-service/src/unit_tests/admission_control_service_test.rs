// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    admission_control_service::{
        AdmissionControlService, SubmitTransactionRequest,
        SubmitTransactionResponse as ProtoSubmitTransactionResponse,
    },
    mocks::local_mock_mempool::LocalMockMempool,
};
use admission_control_proto::{AdmissionControlStatus, SubmitTransactionResponse};
use futures::channel::mpsc;
use libra_crypto::{ed25519::*, test_utils::TEST_SEED};
use libra_mempool_shared_proto::proto::mempool_status::MempoolAddTransactionStatusCode;
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    vm_error::{StatusCode, VMStatus},
};
use rand::SeedableRng;
use std::convert::TryFrom;
use std::sync::Arc;
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

pub fn create_ac_service_for_ut() -> AdmissionControlService<LocalMockMempool, MockVMValidator> {
    let (upstream_proxy_sender, _) = mpsc::unbounded();
    AdmissionControlService::new(
        Some(Arc::new(LocalMockMempool::new())),
        Arc::new(MockStorageReadClient),
        Arc::new(MockVMValidator),
        false,
        upstream_proxy_sender,
    )
}

fn assert_status(response: ProtoSubmitTransactionResponse, status: VMStatus) {
    let rust_resp = SubmitTransactionResponse::try_from(response).unwrap();
    if let Some(resp_ac_status) = rust_resp.ac_status {
        assert_eq!(resp_ac_status, AdmissionControlStatus::Accepted);
    } else {
        let decoded_response = rust_resp.vm_error.unwrap();
        assert_eq!(decoded_response.major_status, status.major_status);
        assert_eq!(decoded_response.sub_status, status.sub_status);
    }
}

#[test]
fn test_submit_txn_inner_vm() {
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let ac_service = create_ac_service_for_ut();
    // create request
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::default();
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST),
    );
    let sender = AccountAddress::new([1; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::INVALID_SIGNATURE));
    let sender = AccountAddress::new([2; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::new(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE),
    );
    let sender = AccountAddress::new([3; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_NEW));
    let sender = AccountAddress::new([4; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD));
    let sender = AccountAddress::new([5; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::TRANSACTION_EXPIRED));
    let sender = AccountAddress::new([6; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::INVALID_AUTH_KEY));
    let sender = AccountAddress::new([8; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::EXECUTED));

    let sender = AccountAddress::new([8; ADDRESS_LENGTH]);
    let test_key = compat::generate_keypair(&mut rng);
    req.signed_txn =
        Some(get_test_signed_txn(sender, 0, keypair.0.clone(), test_key.1.clone(), None).into());
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::new(StatusCode::INVALID_SIGNATURE));
}

#[test]
fn test_submit_txn_inner_mempool() {
    let ac_service = create_ac_service_for_ut();
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::default();
    let keypair = compat::generate_keypair(None);
    let insufficient_balance_add = AccountAddress::new([100; ADDRESS_LENGTH]);
    req.signed_txn = Some(
        get_test_signed_txn(
            insufficient_balance_add,
            0,
            keypair.0.clone(),
            keypair.1.clone(),
            None,
        )
        .into(),
    );
    let response = SubmitTransactionResponse::try_from(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::InsufficientBalance
    );
    let invalid_seq_add = AccountAddress::new([101; ADDRESS_LENGTH]);
    req.signed_txn = Some(
        get_test_signed_txn(
            invalid_seq_add,
            0,
            keypair.0.clone(),
            keypair.1.clone(),
            None,
        )
        .into(),
    );
    let response = SubmitTransactionResponse::try_from(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::InvalidSeqNumber
    );
    let sys_error_add = AccountAddress::new([102; ADDRESS_LENGTH]);
    req.signed_txn = Some(
        get_test_signed_txn(sys_error_add, 0, keypair.0.clone(), keypair.1.clone(), None).into(),
    );
    let response = SubmitTransactionResponse::try_from(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::InvalidUpdate
    );
    let accepted_add = AccountAddress::new([103; ADDRESS_LENGTH]);
    req.signed_txn = Some(
        get_test_signed_txn(accepted_add, 0, keypair.0.clone(), keypair.1.clone(), None).into(),
    );
    let response = SubmitTransactionResponse::try_from(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.ac_status.unwrap(),
        AdmissionControlStatus::Accepted,
    );
    let accepted_add = AccountAddress::new([104; ADDRESS_LENGTH]);
    req.signed_txn =
        Some(get_test_signed_txn(accepted_add, 0, keypair.0.clone(), keypair.1, None).into());
    let response = SubmitTransactionResponse::try_from(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap().code,
        MempoolAddTransactionStatusCode::MempoolIsFull,
    );
}
