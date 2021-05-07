// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::tracing::{observe_block, BlockStage},
    counters,
    state_replication::StateComputer,
};
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_crypto::HashValue;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_mempool::TransactionExclusion;
use diem_types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::TransactionStatus,
};
use futures::{SinkExt, StreamExt};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

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
            info!(
                "Receive ordered block at round {}",
                blocks.last().unwrap().round()
            );
            let executed_blocks: Vec<ExecutedBlock> = blocks
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
