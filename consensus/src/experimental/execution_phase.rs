// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::StateComputer;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;

pub struct ExecutionPhase {
    executor: Arc<dyn StateComputer>,
    receive_channel: channel::Receiver<(Vec<Block>, LedgerInfoWithSignatures)>,
    notify_channel: channel::Sender<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
}

impl ExecutionPhase {
    pub fn new(
        executor: Arc<dyn StateComputer>,
        receive_channel: channel::Receiver<(Vec<Block>, LedgerInfoWithSignatures)>,
        notify_channel: channel::Sender<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    ) -> Self {
        Self {
            executor,
            receive_channel,
            notify_channel,
        }
    }

    pub async fn start(mut self) {
        while let Some((blocks, li)) = self.receive_channel.next().await {
            let executed_blocks = blocks
                .into_iter()
                .map(|block| {
                    let executed_result = self.executor.compute(&block, HashValue::zero()).unwrap();
                    ExecutedBlock::new(block, executed_result)
                })
                .collect();
            if let Err(e) = self.notify_channel.send((executed_blocks, li)).await {
                error!("{:?}", e);
            }
        }
    }
}
