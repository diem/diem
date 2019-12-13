// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    proto::mempool::Mempool,
    OP_COUNTERS,
};
use futures::Future;
use grpc_helpers::{create_grpc_invalid_arg_status, default_reply_error_logger};
use libra_logger::prelude::*;
use libra_metrics::counters::SVC_COUNTERS;
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

impl Mempool for MempoolService {
    fn add_transaction_with_validation(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        req: crate::proto::mempool::AddTransactionWithValidationRequest,
        sink: ::grpcio::UnarySink<crate::proto::mempool::AddTransactionWithValidationResponse>,
    ) {
        trace!("[GRPC] Mempool::add_transaction_with_validation");
        let _timer = SVC_COUNTERS.req(&ctx);
        let mut success = true;
        let proto_transaction = req.transaction.unwrap_or_else(Default::default);
        match SignedTransaction::try_from(proto_transaction) {
            Err(e) => {
                success = false;
                ctx.spawn(
                    sink.fail(create_grpc_invalid_arg_status(
                        "add_transaction_with_validation",
                        e,
                    ))
                    .map_err(default_reply_error_logger),
                );
            }
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

                let mut response =
                    crate::proto::mempool::AddTransactionWithValidationResponse::default();
                response.status = Some(insertion_result.into());
                ctx.spawn(sink.success(response).map_err(default_reply_error_logger))
            }
        }
        SVC_COUNTERS.resp(&ctx, success);
    }

    fn get_block(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        req: super::proto::mempool::GetBlockRequest,
        sink: ::grpcio::UnarySink<super::proto::mempool::GetBlockResponse>,
    ) {
        trace!("[GRPC] Mempool::get_block");
        let _timer = SVC_COUNTERS.req(&ctx);

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
        let mut response = crate::proto::mempool::GetBlockResponse::default();
        response.block = Some(block);
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger));
        SVC_COUNTERS.resp(&ctx, true);
    }

    fn commit_transactions(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        req: crate::proto::mempool::CommitTransactionsRequest,
        sink: ::grpcio::UnarySink<crate::proto::mempool::CommitTransactionsResponse>,
    ) {
        trace!("[GRPC] Mempool::commit_transaction");
        let _timer = SVC_COUNTERS.req(&ctx);
        OP_COUNTERS.inc_by("commit_transactions.requested", req.transactions.len());
        let mut pool = self
            .core_mempool
            .lock()
            .expect("[update status] acquire mempool lock");
        for transaction in &req.transactions {
            if let Ok(address) = AccountAddress::try_from(&transaction.sender[..]) {
                let sequence_number = transaction.sequence_number;
                pool.remove_transaction(&address, sequence_number, transaction.is_rejected);
            }
        }
        let block_timestamp_usecs = req.block_timestamp_usecs;
        if block_timestamp_usecs > 0 {
            pool.gc_by_expiration_time(Duration::from_micros(block_timestamp_usecs));
        }
        let response = crate::proto::mempool::CommitTransactionsResponse::default();
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger));
        SVC_COUNTERS.resp(&ctx, true);
    }

    fn health_check(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        _req: crate::proto::mempool::HealthCheckRequest,
        sink: ::grpcio::UnarySink<crate::proto::mempool::HealthCheckResponse>,
    ) {
        trace!("[GRPC] Mempool::health_check");
        let pool = self
            .core_mempool
            .lock()
            .expect("[health_check] acquire mempool lock");
        let mut response = crate::proto::mempool::HealthCheckResponse::default();
        response.is_healthy = pool.health_check();
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger));
    }
}
