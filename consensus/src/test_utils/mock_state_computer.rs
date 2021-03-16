// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::StateSyncError, state_replication::StateComputer, test_utils::mock_storage::MockStorage,
};
use anyhow::{format_err, Result};
use consensus_types::{block::Block, common::Payload};
use diem_crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, HashValue};
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
        let result = StateComputeResult::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            vec![],
            0,
            None,
            vec![],
            vec![],
        );
        Ok(result)
    }

    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        commit: LedgerInfoWithSignatures,
    ) -> Result<(), Error> {
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());

        // mock sending commit notif to state sync
        let mut txns = vec![];
        for block_id in block_ids {
            let mut payload = self
                .block_cache
                .lock()
                .remove(&block_id)
                .ok_or_else(|| format_err!("Cannot find block"))?;
            txns.append(&mut payload);
        }
        // they may fail during shutdown
        let _ = self.state_sync_client.unbounded_send(txns);

        let _ = self.commit_callback.unbounded_send(commit);
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

pub struct EmptyStateComputer {
    version: Mutex<HashMap<HashValue, u64>>,
}

impl EmptyStateComputer {
    pub fn new() -> Self {
        Self {
            version: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for EmptyStateComputer {
    fn compute(
        &self,
        block: &Block,
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let new_version = *self.version.lock().get(&parent_block_id).unwrap_or(&0)
            + block.payload().unwrap_or(&vec![]).len() as u64;
        self.version.lock().insert(block.id(), new_version);
        Ok(StateComputeResult::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            new_version,
            vec![],
            0,
            None,
            vec![],
            vec![],
        ))
    }

    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        _commit: LedgerInfoWithSignatures,
    ) -> Result<(), Error> {
        let mut l = self.version.lock();
        for id in block_ids {
            l.remove(&id);
        }
        Ok(())
    }

    async fn sync_to(&self, _commit: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        Ok(())
    }
}
