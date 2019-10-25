// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::test_utils::{mock_storage::MockStorage, TestPayload},
    state_replication::StateComputer,
};
use consensus_types::block::Block;
use executor::{ExecutedTrees, ProcessedVMOutput};
use failure::Result;
use futures::{channel::mpsc, future, Future, FutureExt};
use libra_logger::prelude::*;
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use libra_types::validator_set::ValidatorSet;
use std::{pin::Pin, sync::Arc};
use termion::color::*;

pub struct MockStateComputer {
    commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
    consensus_db: Arc<MockStorage<TestPayload>>,
    reconfig: Option<ValidatorSet>,
}

impl MockStateComputer {
    pub fn new(
        commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
        consensus_db: Arc<MockStorage<TestPayload>>,
        reconfig: Option<ValidatorSet>,
    ) -> Self {
        MockStateComputer {
            commit_callback,
            consensus_db,
            reconfig,
        }
    }
}

impl StateComputer for MockStateComputer {
    type Payload = Vec<usize>;
    fn compute(
        &self,
        _block: &Block<Self::Payload>,
        _parent_executed_trees: ExecutedTrees,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedVMOutput>> + Send>> {
        future::ok(ProcessedVMOutput::new(
            vec![],
            ExecutedTrees::new_empty(),
            self.reconfig.clone(),
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

    fn sync_to_deprecated(
        &self,
        commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>> {
        debug!(
            "{}Fake sync{} to block id {}",
            Fg(Blue),
            Fg(Reset),
            commit.ledger_info().consensus_block_id()
        );
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());
        self.commit_callback
            .unbounded_send(commit.clone())
            .expect("Fail to notify about sync");
        async { Ok(true) }.boxed()
    }

    fn state_sync(
        &self,
        _target: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<LedgerInfoWithSignatures>> + Send>> {
        async {
            bail!("Unimplemented!");
        }
            .boxed()
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
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedVMOutput>> + Send>> {
        future::ok(ProcessedVMOutput::new(
            vec![],
            ExecutedTrees::new_empty(),
            None,
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

    fn sync_to_deprecated(
        &self,
        _commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>> {
        async { Ok(true) }.boxed()
    }

    fn state_sync(
        &self,
        _target: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<LedgerInfoWithSignatures>> + Send>> {
        async {
            bail!("Unimplemented!");
        }
            .boxed()
    }

    fn committed_trees(&self) -> ExecutedTrees {
        ExecutedTrees::new_empty()
    }
}
