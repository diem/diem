// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::StateSyncError,
    state_replication::{StateComputer, StateComputerCommitCallBackType},
    test_utils::mock_storage::MockStorage,
};
use anyhow::{format_err, Result};
use consensus_types::{block::Block, common::Payload, executed_block::ExecutedBlock};
use diem_crypto::HashValue;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{Error, StateComputeResult};
use futures::channel::mpsc;
use std::{collections::HashMap, sync::Arc};
use termion::color::*;

pub struct MockStateComputer {
    state_sync_client: mpsc::UnboundedSender<Payload>,
    commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
    consensus_db: Arc<MockStorage>,
    block_cache: Mutex<HashMap<HashValue, Payload>>,
}

impl MockStateComputer {
    pub fn new(
        state_sync_client: mpsc::UnboundedSender<Payload>,
        commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
        consensus_db: Arc<MockStorage>,
    ) -> Self {
        MockStateComputer {
            state_sync_client,
            commit_callback,
            consensus_db,
            block_cache: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for MockStateComputer {
    fn compute(
        &self,
        block: &Block,
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        self.block_cache
            .lock()
            .insert(block.id(), block.payload().unwrap_or(&vec![]).clone());
        let result = StateComputeResult::new_dummy();
        Ok(result)
    }

    async fn commit(
        &self,
        blocks: &[Arc<ExecutedBlock>],
        commit: LedgerInfoWithSignatures,
        call_back: StateComputerCommitCallBackType,
    ) -> Result<(), Error> {
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());

        // mock sending commit notif to state sync
        let mut txns = vec![];
        for block in blocks {
            let mut payload = self
                .block_cache
                .lock()
                .remove(&block.id())
                .ok_or_else(|| format_err!("Cannot find block"))?;
            txns.append(&mut payload);
        }
        // they may fail during shutdown
        let _ = self.state_sync_client.unbounded_send(txns);

        let _ = self.commit_callback.unbounded_send(commit.clone());

        call_back(blocks, commit);

        Ok(())
    }

    async fn sync_to(&self, commit: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
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
}

pub struct EmptyStateComputer;

#[async_trait::async_trait]
impl StateComputer for EmptyStateComputer {
    fn compute(
        &self,
        _block: &Block,
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        Ok(StateComputeResult::new_dummy())
    }

    async fn commit(
        &self,
        _blocks: &[Arc<ExecutedBlock>],
        _commit: LedgerInfoWithSignatures,
        _call_back: StateComputerCommitCallBackType,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn sync_to(&self, _commit: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        Ok(())
    }
}

pub struct RandomComputeResultStateComputer {
    random_compute_result_root_hash: HashValue,
}

impl RandomComputeResultStateComputer {
    pub fn new() -> Self {
        Self {
            random_compute_result_root_hash: HashValue::random(),
        }
    }

    pub fn get_root_hash(&self) -> HashValue {
        self.random_compute_result_root_hash
    }
}

#[async_trait::async_trait]
impl StateComputer for RandomComputeResultStateComputer {
    fn compute(
        &self,
        _block: &Block,
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        Ok(StateComputeResult::new_dummy_with_root_hash(
            self.random_compute_result_root_hash,
        ))
    }

    async fn commit(
        &self,
        _blocks: &[Arc<ExecutedBlock>],
        _commit: LedgerInfoWithSignatures,
        _call_back: StateComputerCommitCallBackType,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn sync_to(&self, _commit: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        Ok(())
    }
}
