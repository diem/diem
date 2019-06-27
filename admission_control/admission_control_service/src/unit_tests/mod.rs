// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use mempool::proto::{
    mempool::{
        AddTransactionWithValidationRequest, AddTransactionWithValidationResponse,
        HealthCheckRequest, HealthCheckResponse,
    },
    mempool_client::MempoolClientTrait,
    shared::mempool_status::MempoolAddTransactionStatus,
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
pub const ADDRESS_MOCKMEMPOOL_INSUFFICIENT_BALANCE: u8 = 100;
pub const ADDRESS_MOCMEMPOOL_INVALID_SEQUENCE: u8 = 101;
pub const ADDRESS_MOCKMEMPOOL_SYS_ERROR: u8 = 102;
pub const ADDRESS_MOCKMEMPOOL_ACCEPTED: u8 = 103;
pub const ADDRESS_MOCKMEMPOOL_FULL: u8 = 104;

impl MempoolClientTrait for LocalMockMempool {
    fn add_transaction_with_validation(
        &self,
        req: &AddTransactionWithValidationRequest,
    ) -> ::grpcio::Result<AddTransactionWithValidationResponse> {
        let mut resp = AddTransactionWithValidationResponse::new();
        let insufficient_balance_add = [ADDRESS_MOCKMEMPOOL_INSUFFICIENT_BALANCE; ADDRESS_LENGTH];
        let invalid_seq_add = [ADDRESS_MOCMEMPOOL_INVALID_SEQUENCE; ADDRESS_LENGTH];
        let sys_error_add = [ADDRESS_MOCKMEMPOOL_SYS_ERROR; ADDRESS_LENGTH];
        let accepted_add = [ADDRESS_MOCKMEMPOOL_ACCEPTED; ADDRESS_LENGTH];
        let mempool_full = [ADDRESS_MOCKMEMPOOL_FULL; ADDRESS_LENGTH];
        let signed_txn = SignedTransaction::from_proto(req.get_signed_txn().clone()).unwrap();
        let sender = signed_txn.sender();
        if sender.as_ref() == insufficient_balance_add {
            resp.set_status(MempoolAddTransactionStatus::InsufficientBalance);
        } else if sender.as_ref() == invalid_seq_add {
            resp.set_status(MempoolAddTransactionStatus::InvalidSeqNumber);
        } else if sender.as_ref() == sys_error_add {
            resp.set_status(MempoolAddTransactionStatus::InvalidUpdate);
        } else if sender.as_ref() == accepted_add {
            resp.set_status(MempoolAddTransactionStatus::Valid);
        } else if sender.as_ref() == mempool_full {
            resp.set_status(MempoolAddTransactionStatus::MempoolIsFull);
        }
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
