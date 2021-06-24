// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::{Receiver, Sender};
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_types::ledger_info::LedgerInfoWithSignatures;
use execution_correctness::ExecutionCorrectness;
use executor_types::Error as ExecutionError;
use futures::{SinkExt, StreamExt};

/// [ This class is used when consensus.decoupled = true ]
/// ExecutionPhase is a singleton that receives ordered blocks from
/// the ordering state computer and execute them. After the execution is done,
/// ExecutionPhase sends the ordered blocks to the commit phase.
pub struct ExecutionPhase {
    executor_channel_recv: Receiver<(Vec<Block>, LedgerInfoWithSignatures)>,
    execution_correctness_client: Box<dyn ExecutionCorrectness + Send + Sync>,
    commit_channel_send: Sender<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
}

impl ExecutionPhase {
    pub fn new(
        executor_channel_recv: Receiver<(Vec<Block>, LedgerInfoWithSignatures)>,
        execution_correctness_client: Box<dyn ExecutionCorrectness + Send + Sync>,
        commit_channel_send: Sender<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    ) -> Self {
        Self {
            executor_channel_recv,
            execution_correctness_client,
            commit_channel_send,
        }
    }

    pub async fn start(mut self) {
        // main loop
        while let Some((vecblock, ledger_info)) = self.executor_channel_recv.next().await {
            // execute the blocks with execution_correctness_client
            let executed_blocks: Vec<ExecutedBlock> = vecblock
                .into_iter()
                .map(|b| {
                    let state_compute_result = self
                        .execution_correctness_client
                        .execute_block(b.clone(), b.parent_id())
                        .unwrap();
                    ExecutedBlock::new(b, state_compute_result)
                })
                .collect();
            // TODO: add error handling.

            // pass the executed blocks into the commit phase
            self.commit_channel_send
                .send((executed_blocks, ledger_info))
                .await
                .map_err(|e| ExecutionError::InternalError {
                    error: e.to_string(),
                })
                .unwrap();
        }
    }
}
