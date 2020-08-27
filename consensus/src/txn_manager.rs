// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::TxnManager;
use anyhow::{format_err, Result};
use consensus_types::{block::Block, common::Payload};
use executor_types::StateComputeResult;
use futures::channel::{mpsc, oneshot};
use itertools::Itertools;
use libra_mempool::{
    CommittedTransaction, ConsensusRequest, ConsensusResponse, TransactionExclusion,
};
use libra_metrics::monitor;
use libra_trace::prelude::*;
use libra_types::transaction::TransactionStatus;
use std::time::Duration;
use tokio::time::timeout;

/// Proxy interface to mempool
#[derive(Clone)]
pub struct MempoolProxy {
    consensus_to_mempool_sender: mpsc::Sender<ConsensusRequest>,
}

impl MempoolProxy {
    pub fn new(consensus_to_mempool_sender: mpsc::Sender<ConsensusRequest>) -> Self {
        Self {
            consensus_to_mempool_sender,
        }
    }
}

#[async_trait::async_trait]
impl TxnManager for MempoolProxy {
    async fn pull_txns(&self, max_size: u64, exclude_payloads: Vec<&Payload>) -> Result<Payload> {
        // intentional bug
        return Ok(vec![]);
        let mut exclude_txns = vec![];
        for payload in exclude_payloads {
            for transaction in payload {
                exclude_txns.push(TransactionExclusion {
                    sender: transaction.sender(),
                    sequence_number: transaction.sequence_number(),
                });
            }
        }
        let (callback, callback_rcv) = oneshot::channel();
        let req = ConsensusRequest::GetBlockRequest(max_size, exclude_txns, callback);
        // send to shared mempool
        self.consensus_to_mempool_sender.clone().try_send(req)?;
        // wait for response
        match monitor!(
            "pull_txn",
            timeout(Duration::from_secs(1), callback_rcv).await
        ) {
            Err(_) => Err(format_err!(
                "[consensus] did not receive GetBlockResponse on time"
            )),
            Ok(resp) => match resp?? {
                ConsensusResponse::GetBlockResponse(txns) => Ok(txns),
                _ => Err(format_err!(
                    "[consensus] did not receive expected GetBlockResponse"
                )),
            },
        }
    }

    // Consensus notifies mempool of executed transactions
    async fn notify(&self, block: &Block, compute_results: &StateComputeResult) -> Result<()> {
        let mut rejected_txns = vec![];
        let txns = match block.payload() {
            Some(txns) => txns,
            None => return Ok(()),
        };
        // skip the block metadata txn result
        for (txn, status) in txns
            .iter()
            .zip_eq(compute_results.compute_status().iter().skip(1))
        {
            if let TransactionStatus::Discard(_) = status {
                rejected_txns.push(CommittedTransaction {
                    sender: txn.sender(),
                    sequence_number: txn.sequence_number(),
                });
            }
        }

        if rejected_txns.is_empty() {
            return Ok(());
        }

        let (callback, callback_rcv) = oneshot::channel();
        let req = ConsensusRequest::RejectNotification(rejected_txns, callback);

        // send to shared mempool
        self.consensus_to_mempool_sender.clone().try_send(req)?;

        if let Err(e) = monitor!(
            "notify_mempool",
            timeout(Duration::from_secs(1), callback_rcv).await
        ) {
            Err(format_err!("[consensus] txn manager did not receive ACK for commit notification sent to mempool on time: {:?}", e))
        } else {
            Ok(())
        }
    }

    fn trace_transactions(&self, block: &Block) {
        if let Some(txns) = block.payload() {
            for txn in txns.iter() {
                trace_edge!("pull_txns", {"txn", txn.sender(), txn.sequence_number()}, {"block", block.id()});
            }
        };
    }
}
