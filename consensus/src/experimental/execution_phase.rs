// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::commit_phase::CommitChannelType,
    state_replication::{StateComputer, StateComputerCommitCallBackType},
};
use channel::{Receiver, Sender};
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::Error as ExecutionError;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;

/// [ This class is used when consensus.decoupled = true ]
/// ExecutionPhase is a singleton that receives ordered blocks from
/// the ordering state computer and execute them. After the execution is done,
/// ExecutionPhase sends the ordered blocks to the commit phase.
///

pub type ExecutionChannelType = (
    Vec<Block>,
    LedgerInfoWithSignatures,
    StateComputerCommitCallBackType,
);

pub struct ExecutionPhase {
    executor_channel_recv: Receiver<ExecutionChannelType>,
    execution_proxy: Arc<dyn StateComputer>,
    commit_channel_send: Sender<CommitChannelType>,
}

impl ExecutionPhase {
    pub fn new(
        executor_channel_recv: Receiver<ExecutionChannelType>,
        execution_proxy: Arc<dyn StateComputer>,
        commit_channel_send: Sender<CommitChannelType>,
    ) -> Self {
        Self {
            executor_channel_recv,
            execution_proxy,
            commit_channel_send,
        }
    }

    pub async fn start(mut self) {
        // main loop
        while let Some((vecblock, ledger_info, callback)) = self.executor_channel_recv.next().await
        {
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
            self.commit_channel_send
                .send((executed_blocks, ledger_info, callback))
                .await
                .map_err(|e| ExecutionError::InternalError {
                    error: e.to_string(),
                })
                .unwrap();
        }
    }
}
