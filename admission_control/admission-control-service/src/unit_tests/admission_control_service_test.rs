// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_proto::{
    proto::admission_control::{
        admission_control_server::AdmissionControl, submit_transaction_response::Status,
        SubmitTransactionRequest, SubmitTransactionResponse as ProtoSubmitTransactionResponse,
    },
    AdmissionControlStatus, SubmitTransactionResponse,
};
use anyhow::Result;
use futures::executor::block_on;
use libra_crypto::{ed25519::Ed25519PrivateKey, test_utils::TEST_SEED, PrivateKey, Uniform};
use libra_types::{
    account_address::AccountAddress,
    mempool_status::MempoolStatusCode,
    proto::types::{
        MempoolStatus, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
        VmStatus as VmStatusProto,
    },
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::SignedTransaction,
    vm_error::{StatusCode, VMStatus},
};
use rand::SeedableRng;
use std::convert::TryFrom;
use tonic::Request;
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

fn submit_transaction(
    sender: AccountAddress,
    ac_service: &MockAdmissionControlService,
    are_keys_valid: bool,
) -> SubmitTransactionResponse {
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key = if are_keys_valid {
        private_key.public_key()
    } else {
        Ed25519PrivateKey::generate(&mut rng).public_key()
    };
    let mut req = SubmitTransactionRequest::default();
    req.transaction = Some(get_test_signed_txn(sender, 0, &private_key, public_key, None).into());
    SubmitTransactionResponse::try_from(
        block_on(ac_service.submit_transaction(Request::new(req)))
            .unwrap()
            .into_inner(),
    )
    .unwrap()
}

fn test_vm_status_submission(
    account_address: AccountAddress,
    ac_service: &MockAdmissionControlService,
    status: StatusCode,
    are_keys_valid: bool,
) {
    let response = submit_transaction(account_address, ac_service, are_keys_valid);
    if let Some(resp_ac_status) = response.ac_status {
        assert_eq!(resp_ac_status, AdmissionControlStatus::Accepted);
    } else {
        let decoded_response = response.vm_error.unwrap();
        assert_eq!(
            decoded_response.major_status,
            VMStatus::new(status).major_status
        );
        assert_eq!(
            decoded_response.sub_status,
            VMStatus::new(status).sub_status
        );
    }
}

fn test_mempool_status_submission(
    account_address: AccountAddress,
    ac_service: &MockAdmissionControlService,
    status: Option<MempoolStatusCode>,
) {
    let response = submit_transaction(account_address, ac_service, true);
    if let Some(mempool_status) = status {
        assert_eq!(response.mempool_error.unwrap().code, mempool_status,);
    } else {
        assert_eq!(
            response.ac_status.unwrap(),
            AdmissionControlStatus::Accepted,
        );
    }
}

struct MockAdmissionControlService {
    mock_vm: MockVMValidator,
}

impl MockAdmissionControlService {
    fn new() -> Self {
        Self {
            mock_vm: MockVMValidator,
        }
    }
}

#[tonic::async_trait]
impl AdmissionControl for MockAdmissionControlService {
    async fn submit_transaction(
        &self,
        request: tonic::Request<SubmitTransactionRequest>,
    ) -> Result<tonic::Response<ProtoSubmitTransactionResponse>, tonic::Status> {
        let req = request.into_inner();
        let transaction = SignedTransaction::try_from(req.clone().transaction.unwrap()).unwrap();

        let mut resp = ProtoSubmitTransactionResponse::default();

        let validation_result = self.mock_vm.validate_transaction(transaction).await;
        match validation_result.as_ref().map(|result| result.status()) {
            Ok(Some(vm_status)) => {
                resp.status = Some(Status::VmStatus(VmStatusProto::from(vm_status)));
            }
            Ok(None) => {
                let mut status = MempoolStatus::default();
                let invalid_seq_add = [101_u8; AccountAddress::LENGTH];
                let sys_error_add = [102_u8; AccountAddress::LENGTH];
                let accepted_add = [103_u8; AccountAddress::LENGTH];
                let mempool_full = [104_u8; AccountAddress::LENGTH];
                let transaction = SignedTransaction::try_from(req.transaction.unwrap()).unwrap();
                let sender = transaction.sender();
                let sender_ref = sender.as_ref();
                if sender_ref == accepted_add {
                    status.code = MempoolStatusCode::Accepted.into();
                } else if sender_ref == invalid_seq_add {
                    status.code = MempoolStatusCode::InvalidSeqNumber.into();
                } else if sender_ref == sys_error_add {
                    status.code = MempoolStatusCode::InvalidUpdate.into();
                } else if sender_ref == mempool_full {
                    status.code = MempoolStatusCode::MempoolIsFull.into();
                }
                resp.status = Some(Status::MempoolStatus(status));
            }
            _ => {
                panic!("unexpected vm validation error");
            }
        }

        if let Some(Status::MempoolStatus(status)) = resp.status.clone() {
            let code: u64 = MempoolStatusCode::Accepted.into();
            if status.code == code {
                resp.status = Some(Status::AcStatus(AdmissionControlStatus::Accepted.into()));
            }
        }

        Ok(tonic::Response::new(resp))
    }

    async fn update_to_latest_ledger(
        &self,
        _request: tonic::Request<UpdateToLatestLedgerRequest>,
    ) -> Result<tonic::Response<UpdateToLatestLedgerResponse>, tonic::Status> {
        unimplemented!("This method is not needed for this test");
    }
}

#[test]
fn test_submit_txn_inner_vm() {
    let ac_service = MockAdmissionControlService::new();

    let test_cases = vec![
        (0, StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST, true),
        (1, StatusCode::INVALID_SIGNATURE, true),
        (
            2,
            StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
            true,
        ),
        (3, StatusCode::SEQUENCE_NUMBER_TOO_NEW, true),
        (4, StatusCode::SEQUENCE_NUMBER_TOO_OLD, true),
        (5, StatusCode::TRANSACTION_EXPIRED, true),
        (6, StatusCode::INVALID_AUTH_KEY, true),
        (8, StatusCode::EXECUTED, true),
        (8, StatusCode::INVALID_SIGNATURE, false),
    ];
    for (sender_id, status, are_keys_valid) in test_cases.into_iter() {
        test_vm_status_submission(
            AccountAddress::new([sender_id as u8; AccountAddress::LENGTH]),
            &ac_service,
            status,
            are_keys_valid,
        );
    }
}

#[test]
fn test_submit_txn_inner_mempool() {
    let ac_service = MockAdmissionControlService::new();

    let test_cases = vec![
        (101, Some(MempoolStatusCode::InvalidSeqNumber)),
        (102, Some(MempoolStatusCode::InvalidUpdate)),
        (103, None),
        (104, Some(MempoolStatusCode::MempoolIsFull)),
    ];
    for (sender_id, status) in test_cases.into_iter() {
        test_mempool_status_submission(
            AccountAddress::new([sender_id as u8; AccountAddress::LENGTH]),
            &ac_service,
            status,
        );
    }
}
