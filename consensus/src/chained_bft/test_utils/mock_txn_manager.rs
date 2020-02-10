// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{state_replication::TxnManager, txn_manager::MempoolProxy};
use anyhow::Result;
use executor::StateComputeResult;
use futures::channel::mpsc;
use libra_mempool::ConsensusRequest;
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};
use rand::Rng;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub type MockTransaction = usize;

/// Trivial mock: generates MockTransactions on the fly. Each next transaction is the next value.
#[derive(Clone)]
pub struct MockTransactionManager {
    next_val: Arc<AtomicUsize>,
    rejected_txns: Vec<usize>,
    // used non-mocked TxnManager to test interaction with shared mempool
    mempool_proxy: Option<MempoolProxy>,
}

impl MockTransactionManager {
    pub fn new(consensus_to_mempool_sender: Option<mpsc::Sender<ConsensusRequest>>) -> Self {
        let mempool_proxy = match consensus_to_mempool_sender {
            Some(sender) => Some(MempoolProxy::new(sender)),
            None => None,
        };
        Self {
            next_val: Arc::new(AtomicUsize::new(0)),
            rejected_txns: vec![],
            mempool_proxy,
        }
    }
}

// mock transaction status on the fly
fn mock_transaction_status(count: usize) -> Vec<TransactionStatus> {
    let mut statuses = vec![];
    for _ in 0..count {
        let random_status = match rand::thread_rng().gen_range(0, 2) {
            0 => TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
            1 => TransactionStatus::Discard(VMStatus::new(StatusCode::UNKNOWN_VALIDATION_STATUS)),
            _ => unreachable!(),
        };
        statuses.push(random_status);
    }
    statuses
}

#[async_trait::async_trait]
impl TxnManager for MockTransactionManager {
    type Payload = Vec<MockTransaction>;

    /// The returned future is fulfilled with the vector of SignedTransactions
    async fn pull_txns(
        &mut self,
        max_size: u64,
        _exclude_txns: Vec<&Self::Payload>,
    ) -> Result<Self::Payload> {
        let next_value = self.next_val.load(Ordering::SeqCst);
        let upper_bound = next_value + max_size as usize;
        let res = (next_value..upper_bound).collect();
        self.next_val.store(upper_bound, Ordering::SeqCst);
        Ok(res)
    }

    async fn commit_txns(
        &mut self,
        txns: &Self::Payload,
        compute_results: &StateComputeResult,
    ) -> Result<()> {
        if self.mempool_proxy.is_some() {
            let mut compute_results_clone = compute_results.clone();
            compute_results_clone.compute_status = mock_transaction_status(txns.len());
            assert!(self
                .mempool_proxy
                .as_mut()
                .unwrap()
                .commit_txns(&vec![], &compute_results_clone)
                .await
                .is_ok());
        }
        Ok(())
    }

    fn _clone_box(&self) -> Box<dyn TxnManager<Payload = Self::Payload>> {
        Box::new(self.clone())
    }
}
