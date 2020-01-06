// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::test_utils::{mock_storage::MockStorage, TestPayload},
    state_replication::StateComputer,
};
use anyhow::{format_err, Result};
use consensus_types::block::Block;
use consensus_types::executed_block::ExecutedBlock;
use executor::{ExecutedTrees, ProcessedVMOutput};
use futures::channel::mpsc;
use libra_logger::prelude::*;
use libra_types::crypto_proxies::ValidatorSet;
use libra_types::crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeProof};
use std::sync::Arc;
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

#[async_trait::async_trait]
impl StateComputer for MockStateComputer {
    type Payload = Vec<usize>;
    fn compute(
        &self,
        _block: &Block<Self::Payload>,
        _parent_executed_trees: &ExecutedTrees,
        _committed_trees: &ExecutedTrees,
    ) -> Result<ProcessedVMOutput> {
        Ok(ProcessedVMOutput::new(
            vec![],
            ExecutedTrees::new_empty(),
            self.reconfig.clone(),
        ))
    }

    async fn commit(
        &self,
        _blocks: Vec<&ExecutedBlock<Self::Payload>>,
        commit: LedgerInfoWithSignatures,
        _synced_trees: &ExecutedTrees,
    ) -> Result<()> {
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());

        self.commit_callback
            .unbounded_send(commit)
            .expect("Fail to notify about commit.");
        Ok(())
    }

    async fn sync_to(&self, commit: LedgerInfoWithSignatures) -> Result<()> {
        debug!(
            "{}Fake sync{} to block id {}",
            Fg(Blue),
            Fg(Reset),
            commit.ledger_info().consensus_block_id()
        );
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());
        self.commit_callback
            .unbounded_send(commit)
            .expect("Fail to notify about sync");
        Ok(())
    }

    async fn get_epoch_proof(
        &self,
        _start_epoch: u64,
        _end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        Err(format_err!(
            "epoch proof not supported in mock state computer"
        ))
    }
}

pub struct EmptyStateComputer;

#[async_trait::async_trait]
impl StateComputer for EmptyStateComputer {
    type Payload = TestPayload;
    fn compute(
        &self,
        _block: &Block<Self::Payload>,
        _parent_executed_trees: &ExecutedTrees,
        _committed_trees: &ExecutedTrees,
    ) -> Result<ProcessedVMOutput> {
        Ok(ProcessedVMOutput::new(
            vec![],
            ExecutedTrees::new_empty(),
            None,
        ))
    }

    async fn commit(
        &self,
        _blocks: Vec<&ExecutedBlock<Self::Payload>>,
        _commit: LedgerInfoWithSignatures,
        _synced_trees: &ExecutedTrees,
    ) -> Result<()> {
        Ok(())
    }

    async fn sync_to(&self, _commit: LedgerInfoWithSignatures) -> Result<()> {
        Ok(())
    }

    async fn get_epoch_proof(
        &self,
        _start_epoch: u64,
        _end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        Err(format_err!(
            "epoch proof not supported in empty state computer"
        ))
    }
}
