// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::StateComputer;
use anyhow::{Error, Result};
use consensus_types::block::Block;
use executor_types::{BlockExecutor, StateComputeResult};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_metrics::monitor;
use libra_types::{ledger_info::LedgerInfoWithSignatures, transaction::Transaction};
use state_synchronizer::StateSyncClient;
use std::{
    boxed::Box,
    sync::{Arc, Mutex},
};

/// Basic communication with the Execution module;
/// implements StateComputer traits.
pub struct ExecutionProxy {
    execution_correctness_client: Mutex<Box<dyn BlockExecutor + Send + Sync>>,
    synchronizer: Arc<StateSyncClient>,
}

impl ExecutionProxy {
    pub fn new(
        execution_correctness_client: Box<dyn BlockExecutor + Send + Sync>,
        synchronizer: Arc<StateSyncClient>,
    ) -> Self {
        Self {
            execution_correctness_client: Mutex::new(execution_correctness_client),
            synchronizer,
        }
    }

    fn transactions_from_block(block: &Block) -> Vec<Transaction> {
        let mut transactions = vec![Transaction::BlockMetadata(block.into())];
        transactions.extend(
            block
                .payload()
                .unwrap_or(&vec![])
                .iter()
                .map(|txn| Transaction::UserTransaction(txn.clone())),
        );
        transactions
    }
}

#[async_trait::async_trait]
impl StateComputer for ExecutionProxy {
    fn compute(
        &self,
        // The block to be executed.
        block: &Block,
        // The parent block id.
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult> {
        debug!(
            "Executing block {:x}. Parent: {:x}.",
            block.id(),
            block.parent_id(),
        );

        // TODO: figure out error handling for the prologue txn
        monitor!(
            "execute_block",
            self.execution_correctness_client
                .lock()
                .unwrap()
                .execute_block(
                    (block.id(), Self::transactions_from_block(block)),
                    parent_block_id,
                )
                .map_err(Error::from)
        )
    }

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Result<()> {
        let (committed_txns, reconfig_events) = monitor!(
            "commit_block",
            self.execution_correctness_client
                .lock()
                .unwrap()
                .commit_blocks(block_ids, finality_proof)?
        );
        if let Err(e) = monitor!(
            "notify_state_sync",
            self.synchronizer
                .commit(committed_txns, reconfig_events)
                .await
        ) {
            error!("failed to notify state synchronizer: {:?}", e);
        }
        Ok(())
    }

    /// Synchronize to a commit that not present locally.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<()> {
        // Here to start to do state synchronization where ChunkExecutor inside will
        // process chunks and commit to Storage. However, after block execution and
        // commitments, the the sync state of ChunkExecutor may be not up to date so
        // it is required to reset the cache of ChunkExecutor in StateSynchronizer
        // when requested to sync.
        let res = monitor!("sync_to", self.synchronizer.sync_to(target).await);
        // Similarily, after the state synchronization, we have to reset the cache
        // of BlockExecutor to guarantee the latest committed state is up to date.
        self.execution_correctness_client.lock().unwrap().reset()?;
        res
    }
}
