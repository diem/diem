// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_storage::tracing::{observe_block, BlockStage};
use crate::counters;
use crate::state_replication::StateComputer;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use diem_types::transaction::TransactionStatus;
use futures::{SinkExt, StreamExt};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

fn update_counters_for_committed_blocks(blocks_to_commit: &[ExecutedBlock]) {
    for block in blocks_to_commit {
        observe_block(block.block().timestamp_usecs(), BlockStage::COMMITTED);
        let txn_status = block.compute_result().compute_status();
        counters::NUM_TXNS_PER_BLOCK.observe(txn_status.len() as f64);
        counters::COMMITTED_BLOCKS_COUNT.inc();
        counters::LAST_COMMITTED_ROUND.set(block.round() as i64);
        counters::LAST_COMMITTED_VERSION.set(block.compute_result().num_leaves() as i64);

        for status in txn_status.iter() {
            match status {
                TransactionStatus::Keep(_) => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["success"])
                        .inc();
                }
                TransactionStatus::Discard(_) => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["failed"])
                        .inc();
                }
                TransactionStatus::Retry => {
                    counters::COMMITTED_TXNS_COUNT
                        .with_label_values(&["retry"])
                        .inc();
                }
            }
        }
    }
}

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
            // TODO: move this to commit phase once executor supports concurrent execute and commit
            // TODO: aggregate signatures
            let commit_li = LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    executed_blocks.last().unwrap().block_info(),
                    HashValue::zero(),
                ),
                BTreeMap::new(),
            );
            let ids = executed_blocks.iter().map(|b| b.id()).collect();
            self.executor.commit(ids, commit_li).await.unwrap();
            update_counters_for_committed_blocks(&executed_blocks);
            self.back_pressure.store(commit_round, Ordering::SeqCst);

            if let Err(e) = self.notify_channel.send((executed_blocks, li)).await {
                error!("{:?}", e);
            }
        }
    }
}
