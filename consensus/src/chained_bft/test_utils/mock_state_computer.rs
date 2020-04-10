// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::test_utils::{mock_storage::MockStorage, TestPayload},
    state_replication::StateComputer,
};
use anyhow::{format_err, Result};
use consensus_types::block::Block;
use executor_types::StateComputeResult;
use futures::channel::mpsc;
use libra_crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, HashValue};
use libra_logger::prelude::*;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures, on_chain_config::ValidatorSet,
    validator_change::ValidatorChangeProof,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use termion::color::*;

pub struct MockStateComputer {
    state_sync_client: mpsc::UnboundedSender<Vec<usize>>,
    commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
    consensus_db: Arc<MockStorage<TestPayload>>,
    reconfig: Option<ValidatorSet>,
    block_cache: Mutex<HashMap<HashValue, Vec<usize>>>,
}

impl MockStateComputer {
    pub fn new(
        state_sync_client: mpsc::UnboundedSender<Vec<usize>>,
        commit_callback: mpsc::UnboundedSender<LedgerInfoWithSignatures>,
        consensus_db: Arc<MockStorage<TestPayload>>,
        reconfig: Option<ValidatorSet>,
    ) -> Self {
        MockStateComputer {
            state_sync_client,
            commit_callback,
            consensus_db,
            reconfig,
            block_cache: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for MockStateComputer {
    type Payload = Vec<usize>;
    fn compute(
        &self,
        block: &Block<Self::Payload>,
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult> {
        self.block_cache
            .lock()
            .unwrap()
            .insert(block.id(), block.payload().unwrap_or(&vec![]).clone());
        Ok(StateComputeResult::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            self.reconfig.clone(),
            vec![],
            vec![],
        ))
    }

    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        commit: LedgerInfoWithSignatures,
    ) -> Result<()> {
        self.consensus_db
            .commit_to_storage(commit.ledger_info().clone());

        // mock sending commit notif to state sync
        let mut txns = vec![];
        for block_id in block_ids {
            let mut payload = self
                .block_cache
                .lock()
                .unwrap()
                .remove(&block_id)
                .ok_or_else(|| format_err!("Cannot find block"))?;
            txns.append(&mut payload);
        }
        self.state_sync_client
            .unbounded_send(txns)
            .expect("Fail to notify state sync about commit");

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
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult> {
        Ok(StateComputeResult::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            None,
            vec![],
            vec![],
        ))
    }

    async fn commit(
        &self,
        _block_ids: Vec<HashValue>,
        _commit: LedgerInfoWithSignatures,
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
