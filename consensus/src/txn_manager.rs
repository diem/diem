// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    state_replication::{StateComputeResult, TxnManager},
};
use failure::Result;
use futures::{compat::Future01CompatExt, future, Future, FutureExt};
use logger::prelude::*;
use mempool::proto::{
    mempool::{
        CommitTransactionsRequest, CommittedTransaction, GetBlockRequest, TransactionExclusion,
    },
    mempool_grpc::MempoolClient,
};
use proto_conv::FromProto;
use std::{pin::Pin, sync::Arc};
use types::transaction::SignedTransaction;

/// Proxy interface to mempool
pub struct MempoolProxy {
    mempool: Arc<MempoolClient>,
}

impl MempoolProxy {
    pub fn new(mempool: Arc<MempoolClient>) -> Self {
        Self {
            mempool: Arc::clone(&mempool),
        }
    }

    /// Generate mempool commit transactions request given the set of txns and their status
    fn gen_commit_transactions_request(
        txns: &[SignedTransaction],
        compute_result: &StateComputeResult,
        timestamp_usecs: u64,
    ) -> CommitTransactionsRequest {
        let mut all_updates = Vec::new();
        assert_eq!(txns.len(), compute_result.compute_status.len());
        for (txn, success) in txns.iter().zip(compute_result.compute_status.iter()) {
            let mut transaction = CommittedTransaction::new();
            transaction.set_sender(txn.sender().as_ref().to_vec());
            transaction.set_sequence_number(txn.sequence_number());
            if *success {
                counters::SUCCESS_TXNS_COUNT.inc();
                transaction.set_is_rejected(false);
            } else {
                counters::FAILED_TXNS_COUNT.inc();
                transaction.set_is_rejected(true);
            }
            all_updates.push(transaction);
        }
        let mut req = CommitTransactionsRequest::new();
        req.set_transactions(::protobuf::RepeatedField::from_vec(all_updates));
        req.set_block_timestamp_usecs(timestamp_usecs);
        req
    }

    /// Submit the request and return the future, which is fulfilled when the response is received.
    fn submit_commit_transactions_request(
        &self,
        req: CommitTransactionsRequest,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        match self.mempool.commit_transactions_async(&req) {
            Ok(receiver) => async move {
                match receiver.compat().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.into()),
                }
            }
                .boxed(),
            Err(e) => future::err(e.into()).boxed(),
        }
    }
}

impl TxnManager for MempoolProxy {
    type Payload = Vec<SignedTransaction>;

    /// The returned future is fulfilled with the vector of SignedTransactions
    fn pull_txns(
        &self,
        max_size: u64,
        exclude_payloads: Vec<&Self::Payload>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Payload>> + Send>> {
        let mut exclude_txns = vec![];
        for payload in exclude_payloads {
            for signed_txn in payload {
                let mut txn_meta = TransactionExclusion::new();
                txn_meta.set_sender(signed_txn.sender().into());
                txn_meta.set_sequence_number(signed_txn.sequence_number());
                exclude_txns.push(txn_meta);
            }
        }
        let mut get_block_request = GetBlockRequest::new();
        get_block_request.set_max_block_size(max_size);
        get_block_request.set_transactions(::protobuf::RepeatedField::from_vec(exclude_txns));
        match self.mempool.get_block_async(&get_block_request) {
            Ok(receiver) => async move {
                match receiver.compat().await {
                    Ok(mut response) => Ok(response
                        .take_block()
                        .take_transactions()
                        .into_iter()
                        .filter_map(|proto_txn| {
                            match SignedTransaction::from_proto(proto_txn.clone()) {
                                Ok(t) => Some(t),
                                Err(e) => {
                                    security_log(SecurityEvent::InvalidTransactionConsensus)
                                        .error(&e)
                                        .data(&proto_txn)
                                        .log();
                                    None
                                }
                            }
                        })
                        .collect()),
                    Err(e) => Err(e.into()),
                }
            }
                .boxed(),
            Err(e) => future::err(e.into()).boxed(),
        }
    }

    fn commit_txns<'a>(
        &'a self,
        txns: &Self::Payload,
        compute_result: &StateComputeResult,
        // Monotonic timestamp_usecs of committed blocks is used to GC expired transactions.
        timestamp_usecs: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        counters::COMMITTED_BLOCKS_COUNT.inc();
        counters::COMMITTED_TXNS_COUNT.inc_by(txns.len() as i64);
        counters::NUM_TXNS_PER_BLOCK.observe(txns.len() as f64);
        let req =
            Self::gen_commit_transactions_request(txns.as_slice(), compute_result, timestamp_usecs);
        self.submit_commit_transactions_request(req)
    }
}
