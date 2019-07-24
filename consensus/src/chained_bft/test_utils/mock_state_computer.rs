// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{consensus_types::quorum_cert::QuorumCert, test_utils::TestPayload},
    state_replication::{ExecutedState, StateComputeResult, StateComputer},
};
use crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, HashValue};
use failure::Result;
use futures::{channel::mpsc, future, Future, FutureExt};
use logger::prelude::*;
use std::pin::Pin;
use termion::color::*;
use types::crypto_proxies::LedgerInfoWithSignatures;

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
        future::ok(StateComputeResult {
            executed_state: ExecutedState {
                state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
                version: 0,
                validators: None,
            },
            compute_status: vec![],
        })
        .boxed()
    }

    fn commit(
        &self,
        commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        self.commit_callback
            .unbounded_send(commit)
            .expect("Fail to notify about commit.");
        future::ok(()).boxed()
    }

    fn sync_to(&self, commit: QuorumCert) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>> {
        debug!(
            "{}Fake sync{} to block id {}",
            Fg(Blue),
            Fg(Reset),
            commit.ledger_info().ledger_info().consensus_block_id()
        );
        self.commit_callback
            .unbounded_send(commit.ledger_info().clone())
            .expect("Fail to notify about sync");
        async { Ok(true) }.boxed()
    }
}

pub struct EmptyStateComputer;

impl StateComputer for EmptyStateComputer {
    type Payload = TestPayload;
    fn compute(
        &self,
        _parent_id: HashValue,
        _block_id: HashValue,
        _transactions: &Self::Payload,
    ) -> Pin<Box<dyn Future<Output = Result<StateComputeResult>> + Send>> {
        future::ok(StateComputeResult {
            executed_state: ExecutedState {
                state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
                version: 0,
                validators: None,
            },
            compute_status: vec![],
        })
        .boxed()
    }

    fn commit(
        &self,
        _commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        future::ok(()).boxed()
    }

    fn sync_to(&self, _commit: QuorumCert) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>> {
        async { Ok(true) }.boxed()
    }
}
