// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::StateComputer};
use anyhow::{ensure, Result};
use consensus_types::block::Block;
use consensus_types::executed_block::ExecutedBlock;
use executor::{ExecutedTrees, Executor, ProcessedVMOutput};
use libra_logger::prelude::*;
use libra_types::crypto_proxies::ValidatorChangeProof;
use libra_types::{
    crypto_proxies::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Transaction},
};
use state_synchronizer::StateSyncClient;
use std::{
    convert::TryFrom,
    sync::Arc,
    time::{Duration, Instant},
};
use vm_runtime::LibraVM;

/// Basic communication with the Execution module;
/// implements StateComputer traits.
pub struct ExecutionProxy {
    executor: Arc<Executor<LibraVM>>,
    synchronizer: Arc<StateSyncClient>,
}

impl ExecutionProxy {
    pub fn new(executor: Arc<Executor<LibraVM>>, synchronizer: Arc<StateSyncClient>) -> Self {
        Self {
            executor,
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
        // The executed trees after executing the parent block.
        parent_executed_trees: &ExecutedTrees,
        // The last committed trees.
        committed_trees: &ExecutedTrees,
    ) -> Result<ProcessedVMOutput> {
        let pre_execution_instant = Instant::now();
        debug!(
            "Executing block {:x}. Parent: {:x}.",
            block.id(),
            block.parent_id(),
        );

        // TODO: figure out error handling for the prologue txn
        self.executor
            .execute_block(
                Self::transactions_from_block(block),
                parent_executed_trees,
                committed_trees,
            )
            .and_then(|output| {
                let execution_duration = pre_execution_instant.elapsed();
                let num_txns = output.transaction_data().len();
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
                Ok(output)
            })
    }

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        blocks: Vec<&ExecutedBlock<Self::Payload>>,
        finality_proof: LedgerInfoWithSignatures,
        committed_trees: &ExecutedTrees,
    ) -> Result<()> {
        let version = finality_proof.ledger_info().version();
        counters::LAST_COMMITTED_VERSION.set(version as i64);

        let pre_commit_instant = Instant::now();

        let committable_blocks = blocks
            .into_iter()
            .map(|executed_block| {
                (
                    Self::transactions_from_block(executed_block.block()),
                    Arc::clone(executed_block.output()),
                )
            })
            .collect();

        let (committed_txns, reconfiguration_events) =
            self.executor
                .commit_blocks(committable_blocks, finality_proof, committed_trees)?;
        counters::BLOCK_COMMIT_DURATION_S.observe_duration(pre_commit_instant.elapsed());
        if let Err(e) = self
            .synchronizer
            .commit(committed_txns, reconfiguration_events)
            .await
        {
            error!("failed to notify state synchronizer: {:?}", e);
        }
        Ok(())
    }

    /// Synchronize to a commit that not present locally.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<()> {
        counters::STATE_SYNC_COUNT.inc();
        self.synchronizer.sync_to(target).await
    }

    async fn get_epoch_proof(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        self.synchronizer
            .get_epoch_proof(start_epoch, end_epoch)
            .await
    }
}
