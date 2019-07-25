// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use mempool::proto::{
    mempool::{
        AddTransactionWithValidationRequest, AddTransactionWithValidationResponse,
        HealthCheckRequest, HealthCheckResponse,
    },
    mempool_client::MempoolClientTrait,
    shared::mempool_status::{MempoolAddTransactionStatus, MempoolAddTransactionStatusCode},
};
use proto_conv::FromProto;
use std::time::SystemTime;
use types::{account_address::ADDRESS_LENGTH, transaction::SignedTransaction};

// Define a local mempool to use for unit tests here, ignore methods not used by the test
#[derive(Clone)]
pub struct LocalMockMempool {
    created_time: SystemTime,
}

impl LocalMockMempool {
    pub fn new() -> Self {
        Self {
            created_time: SystemTime::now(),
        }
    }
}

impl MempoolClientTrait for LocalMockMempool {
    fn add_transaction_with_validation(
        &self,
        req: &AddTransactionWithValidationRequest,
    ) -> ::grpcio::Result<AddTransactionWithValidationResponse> {
        let mut resp = AddTransactionWithValidationResponse::new();
        let mut status = MempoolAddTransactionStatus::new();
        let insufficient_balance_add = [100_u8; ADDRESS_LENGTH];
        let invalid_seq_add = [101_u8; ADDRESS_LENGTH];
        let sys_error_add = [102_u8; ADDRESS_LENGTH];
        let accepted_add = [103_u8; ADDRESS_LENGTH];
        let mempool_full = [104_u8; ADDRESS_LENGTH];
        let signed_txn = SignedTransaction::from_proto(req.get_signed_txn().clone()).unwrap();
        let sender = signed_txn.sender();
        if sender.as_ref() == insufficient_balance_add {
            status.set_code(MempoolAddTransactionStatusCode::InsufficientBalance);
        } else if sender.as_ref() == invalid_seq_add {
            status.set_code(MempoolAddTransactionStatusCode::InvalidSeqNumber);
        } else if sender.as_ref() == sys_error_add {
            status.set_code(MempoolAddTransactionStatusCode::InvalidUpdate);
        } else if sender.as_ref() == accepted_add {
            status.set_code(MempoolAddTransactionStatusCode::Valid);
        } else if sender.as_ref() == mempool_full {
            status.set_code(MempoolAddTransactionStatusCode::MempoolIsFull);
        }
        resp.set_status(status);
        Ok(resp)
    }
    fn health_check(&self, _req: &HealthCheckRequest) -> ::grpcio::Result<HealthCheckResponse> {
        let mut ret = HealthCheckResponse::new();
        let duration_ms = SystemTime::now()
            .duration_since(self.created_time)
            .unwrap()
            .as_millis();
        ret.set_is_healthy(duration_ms > 500 || duration_ms < 300);
        Ok(ret)
    }
}
