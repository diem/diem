// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::StateComputer};
use anyhow::{ensure, Error, Result};
use consensus_types::block::Block;
use executor_types::{BlockExecutor, StateComputeResult};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Transaction},
};
use state_synchronizer::StateSyncClient;
use std::{
    boxed::Box,
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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

    fn transactions_from_block(block: &Block<Vec<SignedTransaction>>) -> Vec<Transaction> {
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
    type Payload = Vec<SignedTransaction>;

    fn compute(
        &self,
        // The block to be executed.
        block: &Block<Self::Payload>,
        // The parent block id.
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult> {
        let pre_execution_instant = Instant::now();
        debug!(
            "Executing block {:x}. Parent: {:x}.",
            block.id(),
            block.parent_id(),
        );

        // TODO: figure out error handling for the prologue txn
        self.execution_correctness_client
            .lock()
            .unwrap()
            .execute_block(
                (block.id(), Self::transactions_from_block(block)),
                parent_block_id,
            )
            .map_err(Error::from)
            .and_then(|result| {
                let execution_duration = pre_execution_instant.elapsed();
                let num_txns = result.transaction_info_hashes().len();
                ensure!(num_txns > 0, "metadata txn failed to execute");
                counters::BLOCK_EXECUTION_DURATION_S.observe_duration(execution_duration);
                if let Ok(nanos_per_txn) =
                    u64::try_from(execution_duration.as_nanos() / num_txns as u128)
                {
                    // TODO: use duration_float once it's stable
                    // Tracking: https://github.com/rust-lang/rust/issues/54361
                    counters::TXN_EXECUTION_DURATION_S
                        .observe_duration(Duration::from_nanos(nanos_per_txn));
                }
                Ok(result)
            })
    }

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Result<()> {
        let version = finality_proof.ledger_info().version();
        counters::LAST_COMMITTED_VERSION.set(version as i64);

        let pre_commit_instant = Instant::now();

        let (committed_txns, reconfig_events) = self
            .execution_correctness_client
            .lock()
            .unwrap()
            .commit_blocks(block_ids, finality_proof)?;
        counters::BLOCK_COMMIT_DURATION_S.observe_duration(pre_commit_instant.elapsed());
        if let Err(e) = self
            .synchronizer
            .commit(committed_txns, reconfig_events)
            .await
        {
            error!("failed to notify state synchronizer: {:?}", e);
        }
        Ok(())
    }

    /// Synchronize to a commit that not present locally.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<()> {
        counters::STATE_SYNC_COUNT.inc();
        // Here to start to do state synchronization where ChunkExecutor inside will
        // process chunks and commit to Storage. However, after block execution and
        // commitments, the the sync state of ChunkExecutor may be not up to date so
        // it is required to reset the cache of ChunkExecutor in StateSynchronizer
        // when requested to sync.
        let res = self.synchronizer.sync_to(target).await;
        // Similarily, after the state synchronization, we have to reset the cache
        // of BlockExecutor to guarantee the latest committed state is up to date.
        self.execution_correctness_client.lock().unwrap().reset()?;
        res
    }
}
