// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    proto::mempool::{
        mempool_server::Mempool, AddTransactionWithValidationRequest,
        AddTransactionWithValidationResponse, CommitTransactionsRequest,
        CommitTransactionsResponse, GetBlockRequest, GetBlockResponse, HealthCheckRequest,
        HealthCheckResponse,
    },
    OP_COUNTERS,
};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress, proto::types::SignedTransactionsBlock,
    transaction::SignedTransaction,
};
use std::{
    cmp,
    collections::HashSet,
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone)]
pub(crate) struct MempoolService {
    pub(crate) core_mempool: Arc<Mutex<CoreMempool>>,
}

#[tonic::async_trait]
impl Mempool for MempoolService {
    async fn add_transaction_with_validation(
        &self,
        request: tonic::Request<AddTransactionWithValidationRequest>,
    ) -> Result<tonic::Response<AddTransactionWithValidationResponse>, tonic::Status> {
        let req = request.into_inner();
        trace!("[GRPC] Mempool::add_transaction_with_validation");

        let proto_transaction = req.transaction.unwrap_or_else(Default::default);
        match SignedTransaction::try_from(proto_transaction) {
            Err(e) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                e.to_string(),
            )),
            Ok(transaction) => {
                let insertion_result = self
                    .core_mempool
                    .lock()
                    .expect("[add txn] acquire mempool lock")
                    .add_txn(
                        transaction,
                        req.max_gas_cost,
                        req.latest_sequence_number,
                        req.account_balance,
                        TimelineState::NotReady,
                    );

                let mut response = AddTransactionWithValidationResponse::default();
                response.status = Some(insertion_result.into());
                Ok(tonic::Response::new(response))
            }
        }
    }

    async fn get_block(
        &self,
        request: tonic::Request<GetBlockRequest>,
    ) -> Result<tonic::Response<GetBlockResponse>, tonic::Status> {
        let req = request.into_inner();
        trace!("[GRPC] Mempool::get_block");

        let block_size = cmp::max(req.max_block_size, 1);
        OP_COUNTERS.inc_by("get_block.requested", block_size as usize);
        let exclude_transactions: HashSet<TxnPointer> = req
            .transactions
            .iter()
            .map(|t| (AccountAddress::try_from(&t.sender[..]), t.sequence_number))
            .filter(|(address, _)| address.is_ok())
            .map(|(address, seq)| (address.unwrap(), seq))
            .collect();

        let mut txns = self
            .core_mempool
            .lock()
            .expect("[get_block] acquire mempool lock")
            .get_block(block_size, exclude_transactions);

        let transactions = txns.drain(..).map(SignedTransaction::into).collect();

        let mut block = SignedTransactionsBlock::default();
        block.transactions = transactions;
        OP_COUNTERS.inc_by("get_block.returned", block.transactions.len());
        let mut response = GetBlockResponse::default();
        response.block = Some(block);
        Ok(tonic::Response::new(response))
    }

    async fn commit_transactions(
        &self,
        request: tonic::Request<CommitTransactionsRequest>,
    ) -> Result<tonic::Response<CommitTransactionsResponse>, tonic::Status> {
        let request = request.into_inner();
        trace!("[GRPC] Mempool::commit_transaction");
        OP_COUNTERS.inc_by("commit_transactions.requested", request.transactions.len());
        let mut pool = self.core_mempool.lock().unwrap();

        for transaction in &request.transactions {
            if let Ok(address) = AccountAddress::try_from(&transaction.sender[..]) {
                let sequence_number = transaction.sequence_number;
                pool.remove_transaction(&address, sequence_number, transaction.is_rejected);
            }
        }

        let block_timestamp_usecs = request.block_timestamp_usecs;
        if block_timestamp_usecs > 0 {
            pool.gc_by_expiration_time(Duration::from_micros(block_timestamp_usecs));
        }

        Ok(tonic::Response::new(CommitTransactionsResponse::default()))
    }

    async fn health_check(
        &self,
        _request: tonic::Request<HealthCheckRequest>,
    ) -> Result<tonic::Response<HealthCheckResponse>, tonic::Status> {
        trace!("[GRPC] Mempool::health_check");
        let pool = self.core_mempool.lock().unwrap();
        let mut response = HealthCheckResponse::default();
        response.is_healthy = pool.health_check();
        Ok(tonic::Response::new(response))
    }
}
