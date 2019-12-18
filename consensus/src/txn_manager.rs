// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::TxnManager};
use anyhow::Result;
use executor::StateComputeResult;
use libra_logger::prelude::*;
use libra_mempool::proto::mempool::{
    CommitTransactionsRequest, CommittedTransaction, GetBlockRequest, TransactionExclusion,
};
use libra_mempool::proto::mempool_client::MempoolClientWrapper;
use libra_types::transaction::{SignedTransaction, TransactionStatus};
use std::convert::TryFrom;

/// Proxy interface to mempool
#[derive(Clone)]
pub struct MempoolProxy {
    mempool: MempoolClientWrapper,
}

impl MempoolProxy {
    pub fn new(mempool: MempoolClientWrapper) -> Self {
        Self { mempool }
    }

    /// Generate mempool commit transactions request given the set of txns and their status
    fn gen_commit_transactions_request(
        txns: &[SignedTransaction],
        compute_result: &StateComputeResult,
        timestamp_usecs: u64,
    ) -> CommitTransactionsRequest {
        let mut all_updates = Vec::new();
        // we exclude the prologue txn, we probably need a way to ensure this aligns with state_computer
        let status = compute_result.compute_status[1..].to_vec();
        assert_eq!(txns.len(), status.len());
        for (txn, status) in txns.iter().zip(compute_result.compute_status.iter()) {
            let mut transaction = CommittedTransaction::default();
            transaction.sender = txn.sender().as_ref().to_vec();
            transaction.sequence_number = txn.sequence_number();
            match status {
                TransactionStatus::Keep(_) => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["success"])
                        .inc();
                    transaction.is_rejected = false;
                }
                TransactionStatus::Discard(_) => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["failed"])
                        .inc();
                    transaction.is_rejected = true;
                }
            };
            all_updates.push(transaction);
        }
        let mut req = CommitTransactionsRequest::default();
        req.transactions = all_updates;
        req.block_timestamp_usecs = timestamp_usecs;
        req
    }

    /// Submit the request and return the future, which is fulfilled when the response is received.
    async fn submit_commit_transactions_request(
        &mut self,
        request: CommitTransactionsRequest,
    ) -> Result<()> {
        self.mempool.commit_transactions(request).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl TxnManager for MempoolProxy {
    type Payload = Vec<SignedTransaction>;

    /// The returned future is fulfilled with the vector of SignedTransactions
    async fn pull_txns(
        &mut self,
        max_size: u64,
        exclude_payloads: Vec<&Self::Payload>,
    ) -> Result<Self::Payload> {
        let mut exclude_txns = vec![];
        for payload in exclude_payloads {
            for transaction in payload {
                let mut txn_meta = TransactionExclusion::default();
                txn_meta.sender = transaction.sender().into();
                txn_meta.sequence_number = transaction.sequence_number();
                exclude_txns.push(txn_meta);
            }
        }
        let mut get_block_request = GetBlockRequest::default();
        get_block_request.max_block_size = max_size;
        get_block_request.transactions = exclude_txns;
        let response = self.mempool.get_block(get_block_request).await?;
        Ok(response
            .block
            .unwrap_or_else(Default::default)
            .transactions
            .into_iter()
            .filter_map(
                |proto_txn| match SignedTransaction::try_from(proto_txn.clone()) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        security_log(SecurityEvent::InvalidTransactionConsensus)
                            .error(&e)
                            .data(&proto_txn)
                            .log();
                        None
                    }
                },
            )
            .collect())
    }

    async fn commit_txns(
        &mut self,
        txns: &Self::Payload,
        compute_result: &StateComputeResult,
        // Monotonic timestamp_usecs of committed blocks is used to GC expired transactions.
        timestamp_usecs: u64,
    ) -> Result<()> {
        counters::COMMITTED_BLOCKS_COUNT.inc();
        counters::NUM_TXNS_PER_BLOCK.observe(txns.len() as f64);
        let req =
            Self::gen_commit_transactions_request(txns.as_slice(), compute_result, timestamp_usecs);
        self.submit_commit_transactions_request(req).await
    }
}
