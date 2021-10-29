// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::StateSyncError,
    experimental::{
        buffer_manager::{OrderedBlocks, ResetAck, ResetRequest},
        errors::Error,
    },
    state_replication::{StateComputer, StateComputerCommitCallBackType},
};
use anyhow::Result;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{Error as ExecutionError, StateComputeResult};
use fail::fail_point;
use futures::{
    channel::{mpsc::UnboundedSender, oneshot},
    SinkExt,
};
use std::{boxed::Box, sync::Arc};

/// Ordering-only execution proxy
/// implements StateComputer traits.
/// Used only when node_config.validator.consensus.decoupled = true.
pub struct OrderingStateComputer {
    // the channel to pour vectors of blocks into
    // the real execution phase (will be handled in ExecutionPhase).
    executor_channel: UnboundedSender<OrderedBlocks>,
    state_computer_for_sync: Arc<dyn StateComputer>,
    reset_event_channel_tx: UnboundedSender<ResetRequest>,
}

impl OrderingStateComputer {
    pub fn new(
        executor_channel: UnboundedSender<OrderedBlocks>,
        state_computer_for_sync: Arc<dyn StateComputer>,
        reset_event_channel_tx: UnboundedSender<ResetRequest>,
    ) -> Self {
        Self {
            executor_channel,
            state_computer_for_sync,
            reset_event_channel_tx,
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for OrderingStateComputer {
    async fn compute(
        &self,
        // The block to be executed.
        _block: &Block,
        // The parent block id.
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult, ExecutionError> {
        // Return dummy block and bypass the execution phase.
        // This will break the e2e smoke test (for now because
        // no one is actually handling the next phase) if the
        // decoupled execution feature is turned on.
        Ok(StateComputeResult::new_dummy())
    }

    /// Send ordered blocks to the real execution phase through the channel.
    /// A future is fulfilled right away when the blocks are sent into the channel.
    async fn commit(
        &self,
        blocks: &[Arc<ExecutedBlock>],
        finality_proof: LedgerInfoWithSignatures,
        callback: StateComputerCommitCallBackType,
    ) -> Result<(), ExecutionError> {
        assert!(!blocks.is_empty());

        if self
            .executor_channel
            .clone()
            .send(OrderedBlocks {
                ordered_blocks: blocks
                    .iter()
                    .map(|b| (**b).clone())
                    .collect::<Vec<ExecutedBlock>>(),
                ordered_proof: finality_proof,
                callback,
            })
            .await
            .is_err()
        {
            debug!("Failed to send to buffer manager, maybe epoch ends");
        }

        Ok(())
    }

    /// Synchronize to a commit that not present locally.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        fail_point!("consensus::sync_to", |_| {
            Err(anyhow::anyhow!("Injected error in sync_to").into())
        });

        // reset execution phase and commit phase
        let (tx, rx) = oneshot::channel::<ResetAck>();
        self.reset_event_channel_tx
            .clone()
            .send(ResetRequest {
                tx,
                stop: false, // epoch manager is responsible for sending stop request
            })
            .await
            .map_err(|_| Error::ResetDropped)?;
        rx.await.map_err(|_| Error::ResetDropped)?;

        // TODO: handle the sync error, should re-push the ordered blocks to buffer manager
        // when it's reset but sync fails.
        self.state_computer_for_sync.sync_to(target).await?;
        Ok(())
    }
}
