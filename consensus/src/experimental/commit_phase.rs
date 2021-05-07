// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_storage::tracing::{observe_block, BlockStage};
use crate::counters;
use crate::state_replication::StateComputer;
use consensus_types::executed_block::ExecutedBlock;
use diem_crypto::HashValue;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_mempool::TransactionExclusion;
use diem_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use diem_types::transaction::TransactionStatus;
use futures::StreamExt;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{collections::BTreeMap, sync::Arc};

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

pub struct CommitPhase {
    committer: Arc<dyn StateComputer>,
    receive_channel: channel::Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    back_pressure: Arc<AtomicU64>,
    pending_txns: Arc<Mutex<HashSet<TransactionExclusion>>>,
}

impl CommitPhase {
    pub fn new(
        committer: Arc<dyn StateComputer>,
        receive_channel: channel::Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
        back_pressure: Arc<AtomicU64>,
        pending_txns: Arc<Mutex<HashSet<TransactionExclusion>>>,
    ) -> Self {
        Self {
            committer,
            receive_channel,
            back_pressure,
            pending_txns,
        }
    }

    pub async fn start(mut self) {
        while let Some((executed_blocks, _)) = self.receive_channel.next().await {
            info!(
                "Receive executed block at round {}",
                executed_blocks.last().unwrap().round()
            );
            // TODO: move this to commit phase once executor supports concurrent execute and commit
            // TODO: aggregate signatures
            let commit_li = LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    executed_blocks.last().unwrap().block_info(),
                    HashValue::zero(),
                ),
                BTreeMap::new(),
            );
            let commit_round = commit_li.ledger_info().commit_info().round();
            let ids = executed_blocks.iter().map(|b| b.id()).collect();
            self.committer.commit(ids, commit_li).await.unwrap();
            update_counters_for_committed_blocks(&executed_blocks);
            {
                let mut pending_txns = self.pending_txns.lock();
                for block in &executed_blocks {
                    if let Some(payload) = block.payload() {
                        for txn in payload {
                            pending_txns.remove(&TransactionExclusion {
                                sender: txn.sender(),
                                sequence_number: txn.sequence_number(),
                            });
                        }
                    }
                }
            }
            self.back_pressure.store(commit_round, Ordering::SeqCst);
        }
    }
}
