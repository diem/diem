// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::TxnManager;
use anyhow::Result;
use futures::channel::{mpsc, oneshot};
use libra_mempool::{GetBlockRequest, TransactionExclusion};
use libra_types::transaction::SignedTransaction;

/// Proxy interface to mempool
#[derive(Clone)]
pub struct MempoolProxy {
    mempool_channel: mpsc::Sender<GetBlockRequest>,
}

impl MempoolProxy {
    pub fn new(mempool_channel: mpsc::Sender<GetBlockRequest>) -> Self {
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

        let (callback, callback_recv) = oneshot::channel();
        let req = GetBlockRequest {
            max_block_size: max_size,
            transactions: exclude_txns,
            callback,
        };
        // send to shared mempool
        self.mempool_channel.clone().try_send(req)?;
        // wait for response
        let resp = callback_recv.await??;
        Ok(resp.transactions)
    }

    fn _clone_box(&self) -> Box<dyn TxnManager<Payload = Self::Payload>> {
        Box::new(self.clone())
    }
}
