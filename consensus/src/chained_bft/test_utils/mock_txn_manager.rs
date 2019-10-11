// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::TxnManager;
use executor::StateComputeResult;
use failure::Result;
use futures::{channel::mpsc, future, Future, FutureExt, SinkExt};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

pub type MockTransaction = usize;

/// Trivial mock: generates MockTransactions on the fly. Each next transaction is the next value.
pub struct MockTransactionManager {
    next_val: AtomicUsize,
    committed_txns: Arc<RwLock<Vec<MockTransaction>>>,
    commit_receiver: Option<mpsc::Receiver<usize>>,
    commit_sender: mpsc::Sender<usize>,
}

impl MockTransactionManager {
    pub fn new() -> Self {
        let (commit_sender, commit_receiver) = mpsc::channel(1024);
        Self {
            next_val: AtomicUsize::new(0),
            committed_txns: Arc::new(RwLock::new(vec![])),
            commit_receiver: Some(commit_receiver),
            commit_sender,
        }
    }

    pub fn get_committed_txns(&self) -> Vec<usize> {
        self.committed_txns.read().unwrap().clone()
    }

    /// Pulls the receiver out of the manager to let the clients receive notifications about the
    /// commits.
    pub fn take_commit_receiver(&mut self) -> mpsc::Receiver<usize> {
        self.commit_receiver
            .take()
            .expect("The receiver has been already pulled out.")
    }
}

impl TxnManager for MockTransactionManager {
    type Payload = Vec<MockTransaction>;

    /// The returned future is fulfilled with the vector of SignedTransactions
    fn pull_txns(
        &self,
        max_size: u64,
        _exclude_txns: Vec<&Self::Payload>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Payload>> + Send>> {
        let next_value = self.next_val.load(Ordering::SeqCst);
        let upper_bound = next_value + max_size as usize;
        let res = (next_value..upper_bound).collect();
        self.next_val.store(upper_bound, Ordering::SeqCst);
        future::ok(res).boxed()
    }

    fn commit_txns<'a>(
        &'a self,
        txns: &Self::Payload,
        _compute_result: &StateComputeResult,
        _timestamp_usecs: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        let committed_tns = txns.clone();
        let mut commit_sender = self.commit_sender.clone();
        async move {
            for txn in committed_tns {
                self.committed_txns.write().unwrap().push(txn);
            }
            let len = self.committed_txns.read().unwrap().len();
            commit_sender
                .send(len)
                .await
                .expect("Failed to notify about mempool commit");
            Ok(())
        }
            .boxed()
    }
}
