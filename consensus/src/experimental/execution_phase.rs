// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::{Sender, Receiver};
use consensus_types::block::Block;
use diem_infallible::Mutex;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use execution_correctness::ExecutionCorrectness;
use executor_types::Error as ExecutionError;
use consensus_types::executed_block::ExecutedBlock;
use futures::{StreamExt, SinkExt};

pub struct ExecutionPhase {
    executor_channel_recv: Receiver<(Vec<Block>, LedgerInfoWithSignatures)>,
    execution_correctness_client: Mutex<Box<dyn ExecutionCorrectness + Send + Sync>>,
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
            execution_correctness_client: Mutex::new(execution_correctness_client),
            commit_channel_send,
        }
    }

    pub async fn start(mut self) {
        while let Some((vecblock, ledger_info)) = self.executor_channel_recv.next().await {

            let executed_blocks: Vec<ExecutedBlock> = vecblock.into_iter().map(|b| {
                let state_compute_result = self.execution_correctness_client.lock()
                    .execute_block(b.clone(), b.parent_id()).unwrap();
                ExecutedBlock::new(b, state_compute_result)
            }).collect();

            self.commit_channel_send
                .send((executed_blocks, ledger_info))
                .await
                .map_err(|e| ExecutionError::InternalError {
                    error: e.to_string()
                }).unwrap();
        }
    }
}
