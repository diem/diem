// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::TxnManager};
use anyhow::{anyhow, Result};
use executor::StateComputeResult;
use futures::channel::{mpsc, oneshot};
use libra_mempool::{CommittedTransaction, MempoolRequest, MempoolResponse, TransactionExclusion};
use libra_types::transaction::{SignedTransaction, TransactionStatus};

/// Proxy interface to mempool
#[derive(Clone)]
pub struct MempoolProxy {
    mempool_channel: mpsc::Sender<(MempoolRequest, oneshot::Sender<Result<MempoolResponse>>)>,
}

impl MempoolProxy {
    pub fn new(
        mempool_channel: mpsc::Sender<(MempoolRequest, oneshot::Sender<Result<MempoolResponse>>)>,
    ) -> Self {
        Self { mempool_channel }
    }
}

#[async_trait::async_trait]
impl TxnManager for MempoolProxy {
    type Payload = Vec<SignedTransaction>;

    async fn pull_txns(
        &mut self,
        max_size: u64,
        exclude_payloads: Vec<&Self::Payload>,
    ) -> Result<Self::Payload> {
        let mut exclude_txns = vec![];
        for payload in exclude_payloads {
            for transaction in payload {
                exclude_txns.push(TransactionExclusion {
                    sender: transaction.sender(),
                    sequence_number: transaction.sequence_number(),
                });
            }
        }

        let req = MempoolRequest::GetBlockRequest {
            max_block_size: max_size,
            transactions: exclude_txns,
        };
        let (callback_send, callback_recv) = oneshot::channel();

        // send to shared mempool
        self.mempool_channel
            .clone()
            .try_send((req, callback_send))?;

        // wait for response
        let resp = callback_recv.await??;
        if let MempoolResponse::GetBlockResponse { transactions } = resp {
            Ok(transactions)
        } else {
            Err(anyhow!(
                "Consensus did not get GetBlockResponse as expected"
            ))
        }
    }

    async fn commit_txns(
        &mut self,
        txns: &Self::Payload,
        compute_results: &StateComputeResult,
        // Monotonic timestamp_usecs of committed blocks is used to GC expired transactions.
        timestamp_usecs: u64,
    ) -> Result<()> {
        counters::COMMITTED_BLOCKS_COUNT.inc();
        counters::NUM_TXNS_PER_BLOCK.observe(txns.len() as f64);
        let mut txns_commit = vec![];
        for (txn, status) in txns.iter().zip(compute_results.compute_status.iter()) {
            let is_rejected = match status {
                TransactionStatus::Keep(_) => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["success"])
                        .inc();
                    false
                }
                TransactionStatus::Discard(_) => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["failed"])
                        .inc();
                    true
                }
            };
            txns_commit.push(CommittedTransaction {
                sender: txn.sender(),
                sequence_number: txn.sequence_number(),
                is_rejected,
            });
        }

        let req = MempoolRequest::CommitTransactionsRequest {
            transactions: txns_commit,
            block_timestamp_usecs: timestamp_usecs,
        };

        let (callback_send, callback_recv) = oneshot::channel();

        // send to shared mempool
        self.mempool_channel
            .clone()
            .try_send((req, callback_send))?;
        // wait for ACK
        callback_recv.await??;
        Ok(())
    }

    fn _clone_box(&self) -> Box<dyn TxnManager<Payload = Self::Payload>> {
        Box::new(self.clone())
    }
}
