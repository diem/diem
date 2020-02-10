// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::TxnManager;
use anyhow::{format_err, Result};
use executor::StateComputeResult;
use futures::channel::{mpsc, oneshot};
use libra_mempool::{
    CommittedTransaction, ConsensusRequest, ConsensusResponse, TransactionExclusion,
};
use libra_types::transaction::{SignedTransaction, TransactionStatus};
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
        let (callback, callback_rcv) = oneshot::channel();
        let req = ConsensusRequest::GetBlockRequest(max_size, exclude_txns, callback);
        // send to shared mempool
        self.consensus_to_mempool_sender.clone().try_send(req)?;
        // wait for response
        match timeout(Duration::from_secs(1), callback_rcv).await {
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

    // Consensus notifies mempool of committed transactions that were rejected
    async fn commit_txns(
        &mut self,
        txns: &Self::Payload,
        compute_results: &StateComputeResult,
    ) -> Result<()> {
        let mut rejected_txns = vec![];
        for (txn, status) in txns.iter().zip(compute_results.compute_status.iter()) {
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

        if let Err(e) = timeout(Duration::from_secs(1), callback_rcv).await {
            Err(format_err!("[consensus] txn manager did not receive ACK for commit notification sent to mempool on time: {:?}", e))
        } else {
            Ok(())
        }
    }

    fn _clone_box(&self) -> Box<dyn TxnManager<Payload = Self::Payload>> {
        Box::new(self.clone())
    }
}
