// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    admission_control_service::{
        AdmissionControlService, SubmitTransactionRequest,
        SubmitTransactionResponse as ProtoSubmitTransactionResponse,
    },
    unit_tests::{
        LocalMockMempool, ADDRESS_MOCKMEMPOOL_ACCEPTED, ADDRESS_MOCKMEMPOOL_FULL,
        ADDRESS_MOCKMEMPOOL_INSUFFICIENT_BALANCE, ADDRESS_MOCKMEMPOOL_SYS_ERROR,
        ADDRESS_MOCMEMPOOL_INVALID_SEQUENCE,
    },
};
use admission_control_proto::{AdmissionControlStatus, SubmitTransactionResponse};
use crypto::{
    hash::CryptoHash,
    signing::{generate_keypair, sign_message},
};
use mempool::MempoolAddTransactionStatus;
use proto_conv::FromProto;
use protobuf::{Message, UnknownFields};
use std::sync::Arc;
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::RawTransactionBytes,
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};
use vm_validator::mocks::mock_vm_validator::{
    MockVMValidator, ADDRESS_MOCKVALIDATION_DOESNOTEXIST, ADDRESS_MOCKVALIDATION_EXPIRATION_TIME,
    ADDRESS_MOCKVALIDATION_INSUFFICIENT_BALANCE, ADDRESS_MOCKVALIDATION_INVALID_AUTH_KEY,
    ADDRESS_MOCKVALIDATION_INVALID_SIGNATURE, ADDRESS_MOCKVALIDATION_SEQUENCE_NUMBER_TOO_NEW,
    ADDRESS_MOCKVALIDATION_SEQUENCE_NUMBER_TOO_OLD,
};

fn create_ac_service_for_ut() -> AdmissionControlService<LocalMockMempool, MockVMValidator> {
    AdmissionControlService::new(
        Arc::new(LocalMockMempool::new()),
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
    let ac_service = create_ac_service_for_ut();
    // create request
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::new();
    let sender = AccountAddress::new([ADDRESS_MOCKVALIDATION_DOESNOTEXIST; ADDRESS_LENGTH]);
    let keypair = generate_keypair();
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::SendingAccountDoesNotExist(
            "TEST".to_string(),
        )),
    );
    let sender = AccountAddress::new([ADDRESS_MOCKVALIDATION_INVALID_SIGNATURE; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InvalidSignature),
    );
    let sender = AccountAddress::new([ADDRESS_MOCKVALIDATION_INSUFFICIENT_BALANCE; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InsufficientBalanceForTransactionFee),
    );
    let sender =
        AccountAddress::new([ADDRESS_MOCKVALIDATION_SEQUENCE_NUMBER_TOO_NEW; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::SequenceNumberTooNew),
    );
    let sender =
        AccountAddress::new([ADDRESS_MOCKVALIDATION_SEQUENCE_NUMBER_TOO_OLD; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::SequenceNumberTooOld),
    );
    let sender = AccountAddress::new([ADDRESS_MOCKVALIDATION_EXPIRATION_TIME; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::TransactionExpired),
    );
    let sender = AccountAddress::new([ADDRESS_MOCKVALIDATION_INVALID_AUTH_KEY; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        keypair.1,
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
        keypair.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(response, VMStatus::Execution(ExecutionStatus::Executed));

    let sender = AccountAddress::new([8; ADDRESS_LENGTH]);
    let test_key = generate_keypair();
    req.set_signed_txn(get_test_signed_txn(
        sender,
        0,
        keypair.0.clone(),
        test_key.1,
        None,
    ));
    let response = ac_service.submit_transaction_inner(req.clone()).unwrap();
    assert_status(
        response,
        VMStatus::Validation(VMValidationStatus::InvalidSignature),
    );
}

#[test]
fn test_reject_unknown_fields() {
    let ac_service = create_ac_service_for_ut();
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::new();
    let keypair = generate_keypair();
    let sender = AccountAddress::random();
    let mut signed_txn = get_test_signed_txn(sender, 0, keypair.0.clone(), keypair.1, None);
    let mut raw_txn = protobuf::parse_from_bytes::<::types::proto::transaction::RawTransaction>(
        signed_txn.raw_txn_bytes.as_ref(),
    )
    .unwrap();
    let mut unknown_fields = UnknownFields::new();
    unknown_fields.add_fixed32(1, 2);
    raw_txn.unknown_fields = unknown_fields;

    let bytes = raw_txn.write_to_bytes().unwrap();
    let hash = RawTransactionBytes(&bytes).hash();
    let signature = sign_message(hash, &keypair.0).unwrap();

    signed_txn.set_raw_txn_bytes(bytes);
    signed_txn.set_sender_signature(signature.to_compact().to_vec());
    req.set_signed_txn(signed_txn);
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.ac_status.unwrap(),
        AdmissionControlStatus::Rejected
    );
}

#[test]
fn test_submit_txn_inner_mempool() {
    let ac_service = create_ac_service_for_ut();
    let mut req: SubmitTransactionRequest = SubmitTransactionRequest::new();
    let keypair = generate_keypair();
    let insufficient_balance_add =
        AccountAddress::new([ADDRESS_MOCKMEMPOOL_INSUFFICIENT_BALANCE; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        insufficient_balance_add,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap(),
        MempoolAddTransactionStatus::InsufficientBalance,
    );
    let invalid_seq_add =
        AccountAddress::new([ADDRESS_MOCMEMPOOL_INVALID_SEQUENCE; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        invalid_seq_add,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap(),
        MempoolAddTransactionStatus::InvalidSeqNumber,
    );
    let sys_error_add = AccountAddress::new([ADDRESS_MOCKMEMPOOL_SYS_ERROR; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        sys_error_add,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap(),
        MempoolAddTransactionStatus::InvalidUpdate,
    );
    let accepted_add = AccountAddress::new([ADDRESS_MOCKMEMPOOL_ACCEPTED; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        accepted_add,
        0,
        keypair.0.clone(),
        keypair.1,
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
    let full_add = AccountAddress::new([ADDRESS_MOCKMEMPOOL_FULL; ADDRESS_LENGTH]);
    req.set_signed_txn(get_test_signed_txn(
        full_add,
        0,
        keypair.0.clone(),
        keypair.1,
        None,
    ));
    let response = SubmitTransactionResponse::from_proto(
        ac_service.submit_transaction_inner(req.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        response.mempool_error.unwrap(),
        MempoolAddTransactionStatus::MempoolIsFull,
    );
}
