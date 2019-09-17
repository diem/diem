// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::test_utils::{mock_storage::MockStorage, TestPayload},
    state_replication::StateComputer,
};
use consensus_types::block::Block;
use consensus_types::quorum_cert::QuorumCert;
use crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use executor::{
    transaction_block::ProcessedVMOutput, ExecutedState, ExecutedTrees, StateComputeResult,
};
use failure::Result;
use futures::{channel::mpsc, future, Future, FutureExt};
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use logger::prelude::*;
use std::{pin::Pin, sync::Arc};
use termion::color::*;

pub struct MockStateComputer {
    commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
    consensus_db: Arc<MockStorage<TestPayload>>,
}

impl MockStateComputer {
    pub fn new(
        commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
        consensus_db: Arc<MockStorage<TestPayload>>,
    ) -> Self {
        MockStateComputer {
            commit_callback,
            consensus_db,
        }
    }
}

impl StateComputer for MockStateComputer {
    type Payload = Vec<usize>;
    fn compute(
        &self,
        _block: &Block<Self::Payload>,
        _parent_executed_trees: ExecutedTrees,
    ) -> Pin<Box<dyn Future<Output = Result<(ProcessedVMOutput, StateComputeResult)>> + Send>> {
        future::ok((
            ProcessedVMOutput::new(vec![], ExecutedTrees::new_empty()),
            StateComputeResult {
                executed_state: ExecutedState {
                    state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
                    version: 0,
                    validators: None,
                },
                compute_status: vec![],
            },
        ))
        .boxed()
    }

    fn commit(
        &self,
        _blocks: Vec<(Self::Payload, Arc<ProcessedVMOutput>)>,
        commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());

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
        self.consensus_db
            .commit_to_storage(commit.ledger_info().ledger_info().clone());
        self.commit_callback
            .unbounded_send(commit.ledger_info().clone())
            .expect("Fail to notify about sync");
        async { Ok(true) }.boxed()
    }

    fn committed_trees(&self) -> ExecutedTrees {
        ExecutedTrees::new_empty()
    }
}

pub struct EmptyStateComputer;

impl StateComputer for EmptyStateComputer {
    type Payload = TestPayload;
    fn compute(
        &self,
        _block: &Block<Self::Payload>,
        _parent_executed_trees: ExecutedTrees,
    ) -> Pin<Box<dyn Future<Output = Result<(ProcessedVMOutput, StateComputeResult)>> + Send>> {
        future::ok((
            ProcessedVMOutput::new(vec![], ExecutedTrees::new_empty()),
            StateComputeResult {
                executed_state: ExecutedState {
                    state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
                    version: 0,
                    validators: None,
                },
                compute_status: vec![],
            },
        ))
        .boxed()
    }

    fn commit(
        &self,
        _blocks: Vec<(Self::Payload, Arc<ProcessedVMOutput>)>,
        _commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        future::ok(()).boxed()
    }

    fn sync_to(&self, _commit: QuorumCert) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>> {
        async { Ok(true) }.boxed()
    }

    fn committed_trees(&self) -> ExecutedTrees {
        ExecutedTrees::new_empty()
    }
}
