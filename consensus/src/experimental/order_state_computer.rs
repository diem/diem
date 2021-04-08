// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::StateSyncError, state_replication::StateComputer};
use consensus_types::block::Block;
use diem_crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, HashValue};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_mempool::TransactionExclusion;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{Error as ExecutionError, StateComputeResult};
use futures::SinkExt;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// Ordering only proxy, upon commit it passes ordered blocks to next phase.
pub struct OrderProxy {
    notify_channel: channel::Sender<(Vec<Block>, LedgerInfoWithSignatures)>,
    pending_blocks: Mutex<HashMap<HashValue, Block>>,
    pending_txns: Arc<Mutex<HashSet<TransactionExclusion>>>,
}

impl OrderProxy {
    pub fn new(
        notify_channel: channel::Sender<(Vec<Block>, LedgerInfoWithSignatures)>,
        pending_txns: Arc<Mutex<HashSet<TransactionExclusion>>>,
    ) -> Self {
        Self {
            notify_channel,
            pending_blocks: Mutex::new(HashMap::new()),
            pending_txns,
        }
    }
}

#[async_trait::async_trait]
impl StateComputer for OrderProxy {
    fn compute(
        &self,
        block: &Block,
        _parent_block_id: HashValue,
    ) -> Result<StateComputeResult, ExecutionError> {
        self.pending_blocks.lock().insert(block.id(), block.clone());
        let result = StateComputeResult::new(
            HashValue::zero(),
            vec![],
            0,
            vec![],
            0,
            None,
            vec![],
            vec![],
        );
        Ok(result)
    }

    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Result<(), ExecutionError> {
        let blocks: Vec<Block> = {
            let mut map = self.pending_blocks.lock();
            block_ids.iter().map(|id| map.remove(id).unwrap()).collect()
        };
        {
            let mut pending_txns = self.pending_txns.lock();
            for block in &blocks {
                if let Some(payload) = block.payload() {
                    for txn in payload {
                        pending_txns.insert(TransactionExclusion {
                            sender: txn.sender(),
                            sequence_number: txn.sequence_number(),
                        });
                    }
                }
            }
        }
        self.notify_channel
            .clone()
            .send((blocks, finality_proof))
            .await
            .map_err(|e| ExecutionError::InternalError {
                error: e.to_string(),
            })?;
        Ok(())
    }

    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<(), StateSyncError> {
        unimplemented!();
    }
}
