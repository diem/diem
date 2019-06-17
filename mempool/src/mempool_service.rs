// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    proto::mempool_grpc::Mempool,
    OP_COUNTERS,
};
use futures::Future;
use grpc_helpers::{create_grpc_invalid_arg_status, default_reply_error_logger};
use logger::prelude::*;
use metrics::counters::SVC_COUNTERS;
use proto_conv::{FromProto, IntoProto};
use std::{
    cmp,
    collections::HashSet,
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::Duration,
};
use types::{
    account_address::AccountAddress, proto::transaction::SignedTransactionsBlock,
    transaction::SignedTransaction,
};

#[derive(Clone)]
pub(crate) struct MempoolService {
    pub(crate) core_mempool: Arc<Mutex<CoreMempool>>,
}

impl Mempool for MempoolService {
    fn add_transaction_with_validation(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        mut req: crate::proto::mempool::AddTransactionWithValidationRequest,
        sink: ::grpcio::UnarySink<crate::proto::mempool::AddTransactionWithValidationResponse>,
    ) {
        trace!("[GRPC] Mempool::add_transaction_with_validation");
        let _timer = SVC_COUNTERS.req(&ctx);
        let mut success = true;
        let proto_transaction = req.take_signed_txn();
        match SignedTransaction::from_proto(proto_transaction) {
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
                    crate::proto::mempool::AddTransactionWithValidationResponse::new();
                response.set_status(insertion_result.into_proto());
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

        let block_size = cmp::max(req.get_max_block_size(), 1);
        OP_COUNTERS.inc_by("get_block.requested", block_size as usize);
        let exclude_transactions: HashSet<TxnPointer> = req
            .get_transactions()
            .iter()
            .map(|t| (AccountAddress::try_from(t.get_sender()), t.sequence_number))
            .filter(|(address, _)| address.is_ok())
            .map(|(address, seq)| (address.unwrap(), seq))
            .collect();

        let mut txns = self
            .core_mempool
            .lock()
            .expect("[get_block] acquire mempool lock")
            .get_block(block_size, exclude_transactions);

        let transactions = txns.drain(..).map(SignedTransaction::into_proto).collect();

        let mut block = SignedTransactionsBlock::new();
        block.set_transactions(::protobuf::RepeatedField::from_vec(transactions));
        OP_COUNTERS.inc_by("get_block.returned", block.get_transactions().len());
        let mut response = crate::proto::mempool::GetBlockResponse::new();
        response.set_block(block);
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
        OP_COUNTERS.inc_by(
            "commit_transactions.requested",
            req.get_transactions().len(),
        );
        let mut pool = self
            .core_mempool
            .lock()
            .expect("[update status] acquire mempool lock");
        for transaction in req.get_transactions() {
            if let Ok(address) = AccountAddress::try_from(transaction.get_sender()) {
                let sequence_number = transaction.get_sequence_number();
                pool.remove_transaction(&address, sequence_number, transaction.get_is_rejected());
            }
        }
        let block_timestamp_usecs = req.get_block_timestamp_usecs();
        if block_timestamp_usecs > 0 {
            pool.gc_by_expiration_time(Duration::from_micros(block_timestamp_usecs));
        }
        let response = crate::proto::mempool::CommitTransactionsResponse::new();
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
        let mut response = crate::proto::mempool::HealthCheckResponse::new();
        response.set_is_healthy(pool.health_check());
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger));
    }
}
