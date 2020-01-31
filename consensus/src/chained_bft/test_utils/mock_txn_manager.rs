// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::TxnManager;
use anyhow::Result;
use futures::channel::mpsc;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

pub type MockTransaction = usize;

/// Trivial mock: generates MockTransactions on the fly. Each next transaction is the next value.
#[derive(Clone)]
pub struct MockTransactionManager {
    next_val: Arc<AtomicUsize>,
    committed_txns: Arc<RwLock<Vec<MockTransaction>>>,
    commit_sender: mpsc::Sender<usize>,
}

impl MockTransactionManager {
    pub fn new() -> (Self, mpsc::Receiver<usize>) {
        let (commit_sender, commit_receiver) = mpsc::channel(1024);
        (
            Self {
                next_val: Arc::new(AtomicUsize::new(0)),
                committed_txns: Arc::new(RwLock::new(vec![])),
                commit_sender,
            },
            commit_receiver,
        )
    }

    pub fn get_committed_txns(&self) -> Vec<usize> {
        self.committed_txns.read().unwrap().clone()
    }
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

    fn _clone_box(&self) -> Box<dyn TxnManager<Payload = Self::Payload>> {
        Box::new(self.clone())
    }
}
