// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::StateSyncError, state_replication::StateComputer};
use anyhow::Result;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_crypto::{HashValue, hash::ACCUMULATOR_PLACEHOLDER_HASH};
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{Error as ExecutionError, StateComputeResult};
use fail::fail_point;
use std::{boxed::Box, sync::Arc};
use channel::Sender;
use futures::SinkExt;

/// Ordering-only execution proxy. Pours every blocks into the execution phase.
/// implements StateComputer traits.
/// Used only when node_config.validator.consensus.decoupeld = true.
pub struct OrderingStateComputer {
    executor_channel: Sender<(Vec<Block>, LedgerInfoWithSignatures)>,
}

impl OrderingStateComputer {
    pub fn new(
        executor_channel: Sender<(Vec<Block>, LedgerInfoWithSignatures)>,
    ) -> Self {
        Self {
            executor_channel,
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for OrderingStateComputer {
    fn compute(
        &self,
        // The block to be executed.
        _block: &Block,
        // The parent block id.
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult, ExecutionError> {
        // Return dummy block and bypass the execution phase.
        // This will break the e2e smoke test.
        Ok(StateComputeResult::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            vec![],
            0,
            None,
            vec![],
            vec![],
            vec![],
        ))
    }

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        blocks: &[Arc<ExecutedBlock>],
        finality_proof: LedgerInfoWithSignatures,
    ) -> Result<(), ExecutionError> {

        let ordered_block = blocks.iter()
            .map(|b| b.block().clone()).collect();

        self.executor_channel.clone()
            .send((ordered_block, finality_proof))
            .await
            .map_err(|e| ExecutionError::InternalError {
                error: e.to_string()
            })?;
        Ok(())
    }

    /// Synchronize to a commit that not present locally.
    async fn sync_to(&self, _target: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        fail_point!("consensus::sync_to", |_| {
            Err(anyhow::anyhow!("Injected error in sync_to").into())
        });
        unimplemented!();
    }
}
