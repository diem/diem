// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::{commit_phase::CommitChannelType, errors::Error},
    state_replication::{StateComputer, StateComputerCommitCallBackType},
};
use channel::{Receiver, Sender};
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::Error as ExecutionError;
use futures::{channel::oneshot, select, FutureExt, SinkExt, StreamExt};
use std::sync::Arc;

/// [ This class is used when consensus.decoupled = true ]
/// ExecutionPhase is a singleton that receives ordered blocks from
/// the ordering state computer and execute them. After the execution is done,
/// ExecutionPhase sends the ordered blocks to the commit phase.
///

pub type ResetAck = ();
pub fn reset_ack_new() -> ResetAck {}

pub struct ExecutionChannelType(
    pub Vec<Block>,
    pub LedgerInfoWithSignatures,
    pub StateComputerCommitCallBackType,
);

impl std::fmt::Debug for ExecutionChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for ExecutionChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ExecutionChannelType({:?}, {})", self.0, self.1)
    }
}

pub struct ExecutionPhase {
    executor_channel_rx: Receiver<ExecutionChannelType>,
    execution_proxy: Arc<dyn StateComputer>,
    commit_channel_tx: Sender<CommitChannelType>,
    reset_event_channel_rx: Receiver<oneshot::Sender<ResetAck>>,
    commit_phase_reset_event_tx: Sender<oneshot::Sender<ResetAck>>,
}

impl ExecutionPhase {
    pub fn new(
        executor_channel_rx: Receiver<ExecutionChannelType>,
        execution_proxy: Arc<dyn StateComputer>,
        commit_channel_tx: Sender<CommitChannelType>,
        reset_event_channel_rx: Receiver<oneshot::Sender<ResetAck>>,
        commit_phase_reset_event_tx: Sender<oneshot::Sender<ResetAck>>,
    ) -> Self {
        Self {
            executor_channel_rx,
            execution_proxy,
            commit_channel_tx,
            reset_event_channel_rx,
            commit_phase_reset_event_tx,
        }
    }

    pub async fn process_reset_event(
        &mut self,
        reset_event_callback: oneshot::Sender<ResetAck>,
    ) -> anyhow::Result<()> {
        // reset the execution phase

        // notify the commit phase
        let (tx, rx) = oneshot::channel::<ResetAck>();
        self.commit_phase_reset_event_tx.send(tx).await?;
        rx.await?;

        // exhaust the executor channel
        while self.executor_channel_rx.next().now_or_never().is_some() {}

        // activate the callback
        reset_event_callback
            .send(reset_ack_new())
            .map_err(|_| Error::ResetDropped)?;

        Ok(())
    }

    pub async fn start(mut self) {
        // main loop
        loop {
            select! {
                ExecutionChannelType(vecblock, ledger_info, callback) = self.executor_channel_rx.select_next_some() => {
                    // execute the blocks with execution_correctness_client
                    let executed_blocks: Vec<ExecutedBlock> = vecblock
                        .into_iter()
                        .map(|b| {
                            let state_compute_result =
                                self.execution_proxy.compute(&b, b.parent_id()).unwrap();
                            ExecutedBlock::new(b, state_compute_result)
                        })
                        .collect();
                    // TODO: add error handling. Err(Error::BlockNotFound(parent_block_id))

                    // pass the executed blocks into the commit phase
                    self.commit_channel_tx
                        .send(CommitChannelType(executed_blocks, ledger_info, callback))
                        .await
                        .map_err(|e| ExecutionError::InternalError {
                            error: e.to_string(),
                        })
                        .unwrap();
                }
                reset_event_callback = self.reset_event_channel_rx.select_next_some() => {
                    self.process_reset_event(reset_event_callback).await.map_err(|e| ExecutionError::InternalError {
                        error: e.to_string(),
                    })
                    .unwrap();
                }
                complete => break,
            };
        }
    }
}
