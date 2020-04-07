// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::StateComputer};
use anyhow::{ensure, Result};
use consensus_types::block::Block;
use executor::Executor;
use executor_types::StateComputeResult;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Transaction},
    validator_change::ValidatorChangeProof,
};
use libra_vm::LibraVM;
use state_synchronizer::StateSyncClient;
use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Basic communication with the Execution module;
/// implements StateComputer traits.
pub struct ExecutionProxy {
    executor: Arc<Mutex<Executor<LibraVM>>>,
    synchronizer: Arc<StateSyncClient>,
}

impl ExecutionProxy {
    pub fn new(
        executor: Arc<Mutex<Executor<LibraVM>>>,
        synchronizer: Arc<StateSyncClient>,
    ) -> Self {
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
        self.executor
            .lock()
            .unwrap()
            .execute_block(
                (block.id(), Self::transactions_from_block(block)),
                parent_block_id,
            )
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
            .executor
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
