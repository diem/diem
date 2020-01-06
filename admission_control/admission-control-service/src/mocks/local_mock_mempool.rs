// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_mempool::proto::{
    mempool::{
        AddTransactionWithValidationRequest, AddTransactionWithValidationResponse,
        HealthCheckRequest, HealthCheckResponse,
    },
    mempool_client::MempoolClientTrait,
};
use libra_mempool_shared_proto::proto::mempool_status::{
    MempoolAddTransactionStatus, MempoolAddTransactionStatusCode,
};
use libra_types::{account_address::ADDRESS_LENGTH, transaction::SignedTransaction};
use std::convert::TryFrom;
use std::time::SystemTime;

/// Define a local mempool to use for unit tests and fuzzing,
/// ignore methods not used
#[derive(Clone)]
pub struct LocalMockMempool {
    created_time: SystemTime,
}

impl LocalMockMempool {
    /// Creates a new instance of localMockMempool
    pub fn new() -> Self {
        Self {
            created_time: SystemTime::now(),
        }
    }
}

#[tonic::async_trait]
impl MempoolClientTrait for LocalMockMempool {
    async fn add_transaction_with_validation(
        &mut self,
        request: AddTransactionWithValidationRequest,
    ) -> Result<AddTransactionWithValidationResponse, tonic::Status> {
        let mut resp = AddTransactionWithValidationResponse::default();
        let mut status = MempoolAddTransactionStatus::default();
        let insufficient_balance_add = [100_u8; ADDRESS_LENGTH];
        let invalid_seq_add = [101_u8; ADDRESS_LENGTH];
        let sys_error_add = [102_u8; ADDRESS_LENGTH];
        let accepted_add = [103_u8; ADDRESS_LENGTH];
        let mempool_full = [104_u8; ADDRESS_LENGTH];
        let transaction =
            SignedTransaction::try_from(request.transaction.unwrap()).unwrap();
        let sender = transaction.sender();
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
        resp.status = Some(status);
        Ok(resp)
    }

    async fn health_check(
        &mut self,
        _request: HealthCheckRequest,
    ) -> Result<HealthCheckResponse, tonic::Status> {
        let mut ret = HealthCheckResponse::default();
        let duration_ms = SystemTime::now()
            .duration_since(self.created_time)
            .unwrap()
            .as_millis();
        ret.is_healthy = duration_ms > 500 || duration_ms < 300;
        Ok(ret)
    }
}
