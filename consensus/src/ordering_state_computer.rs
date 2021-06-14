// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::StateSyncError, state_replication::StateComputer};
use anyhow::Result;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_crypto::{HashValue, hash::ACCUMULATOR_PLACEHOLDER_HASH};
use diem_infallible::Mutex;
use diem_metrics::monitor;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{Error as ExecutionError, StateComputeResult};
use fail::fail_point;
use std::{boxed::Box, sync::Arc};
use channel::Sender;

/// Basic communication with the Execution module;
/// implements StateComputer traits.
pub struct OrderingStateComputer {
    executor_channel: Sender<(Vec<Block>, HashValue)>,
    local_blocks: Mutex<Vec<Block>>,
}

impl OrderingStateComputer {
    pub fn new(
        executor_channel: Sender<(Vec<Block>, HashValue)>,
        local_blocks: Mutex<Vec<Block>>,
    ) -> Self {
        Self {
            executor_channel,
            local_blocks,
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for OrderingStateComputer {
    fn compute(
        &self,
        // The block to be executed.
        block: &Block,
        // The parent block id.
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, ExecutionError> {
        fail_point!("consensus::compute", |_| {
            Err(ExecutionError::InternalError {
                error: "Injected error in compute".into(),
            })
        });

        self.local_blocks.lock().push(block.clone());

        // Return dummy block and bypass the execution phase.
        // This will break the e2e smoke test.
        monitor!(
            "execute_block",
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
        )

    }

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        blocks: &[Arc<ExecutedBlock>],
        finality_proof: LedgerInfoWithSignatures,
    ) -> Result<(), ExecutionError> {
        unimplemented!();
    }

    /// Synchronize to a commit that not present locally.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        fail_point!("consensus::sync_to", |_| {
            Err(anyhow::anyhow!("Injected error in sync_to").into())
        });
        unimplemented!();
    }
}
