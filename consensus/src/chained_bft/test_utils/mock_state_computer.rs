// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::consensus_types::quorum_cert::QuorumCert,
    state_replication::{StateComputeResult, StateComputer},
};
use crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, HashValue};
use failure::Result;
use futures::{channel::mpsc, Future, FutureExt};
use logger::prelude::*;
use state_synchronizer::SyncStatus;
use std::pin::Pin;
use termion::color::*;
use types::{ledger_info::LedgerInfoWithSignatures, transaction::TransactionListWithProof};

pub struct MockStateComputer {
    commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
}

impl MockStateComputer {
    pub fn new(commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>) -> Self {
        MockStateComputer { commit_callback }
    }
}

impl StateComputer for MockStateComputer {
    type Payload = Vec<usize>;
    fn compute(
        &self,
        _parent_id: HashValue,
        _block_id: HashValue,
        _transactions: &Self::Payload,
    ) -> Pin<Box<dyn Future<Output = Result<StateComputeResult>> + Send>> {
        async move {
            Ok(StateComputeResult {
                new_state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
                compute_status: vec![],
                num_successful_txns: 0,
                validators: None,
            })
        }
            .boxed()
    }

    fn commit(
        &self,
        commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        self.commit_callback
            .unbounded_send(commit)
            .expect("Fail to notify about commit.");
        async { Ok(()) }.boxed()
    }

    fn sync_to(
        &self,
        commit: QuorumCert,
    ) -> Pin<Box<dyn Future<Output = Result<SyncStatus>> + Send>> {
        debug!(
            "{}Fake sync{} to block id {}",
            Fg(Blue),
            Fg(Reset),
            commit.ledger_info().ledger_info().consensus_block_id()
        );
        self.commit_callback
            .unbounded_send(commit.ledger_info().clone())
            .expect("Fail to notify about sync");
        async { Ok(SyncStatus::Finished) }.boxed()
    }

    fn get_chunk(
        &self,
        _: u64,
        _: u64,
        _: u64,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>> {
        async move { Err(format_err!("not implemented")) }.boxed()
    }
}
