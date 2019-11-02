// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::StateComputer};
use consensus_types::block::Block;
use executor::{CommittableBlock, ExecutedTrees, Executor, ProcessedVMOutput};
use failure::Result;
use futures::{Future, FutureExt};
use libra_logger::prelude::*;
use libra_types::crypto_proxies::ValidatorChangeEventWithProof;
use libra_types::{
    crypto_proxies::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Transaction},
};
use state_synchronizer::StateSyncClient;
use std::{
    convert::TryFrom,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};
use vm_runtime::MoveVM;

/// Basic communication with the Execution module;
/// implements StateComputer traits.
pub struct ExecutionProxy {
    executor: Arc<Executor<MoveVM>>,
    synchronizer: Arc<StateSyncClient>,
}

impl ExecutionProxy {
    pub fn new(executor: Arc<Executor<MoveVM>>, synchronizer: Arc<StateSyncClient>) -> Self {
        Self {
            executor,
            synchronizer,
        }
    }
}

impl StateComputer for ExecutionProxy {
    type Payload = Vec<SignedTransaction>;

    fn compute(
        &self,
        // The block to be executed.
        block: &Block<Self::Payload>,
        // The executed trees after executing the parent block.
        parent_executed_trees: ExecutedTrees,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedVMOutput>> + Send>> {
        let pre_execution_instant = Instant::now();
        let execute_future = self.executor.execute_block(
            block
                .payload()
                .unwrap_or(&Self::Payload::default())
                .iter()
                .map(|txn| Transaction::UserTransaction(txn.clone()))
                .collect(),
            parent_executed_trees,
            block.parent_id(),
            block.id(),
        );
        async move {
            match execute_future.await {
                Ok(Ok(output)) => {
                    let execution_duration = pre_execution_instant.elapsed();
                    let num_txns = output.transaction_data().len();
                    if num_txns == 0 {
                        // no txns in that block
                        counters::EMPTY_BLOCK_EXECUTION_DURATION_S
                            .observe_duration(execution_duration);
                    } else {
                        counters::BLOCK_EXECUTION_DURATION_S.observe_duration(execution_duration);
                        if let Ok(nanos_per_txn) =
                            u64::try_from(execution_duration.as_nanos() / num_txns as u128)
                        {
                            // TODO: use duration_float once it's stable
                            // Tracking: https://github.com/rust-lang/rust/issues/54361
                            counters::TXN_EXECUTION_DURATION_S
                                .observe_duration(Duration::from_nanos(nanos_per_txn));
                        }
                    }
                    Ok(output)
                }
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e.into()),
            }
        }
            .boxed()
    }

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    fn commit(
        &self,
        payload_and_output_list: Vec<(Self::Payload, Arc<ProcessedVMOutput>)>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        let version = finality_proof.ledger_info().version();
        counters::LAST_COMMITTED_VERSION.set(version as i64);

        let pre_commit_instant = Instant::now();
        let synchronizer = Arc::clone(&self.synchronizer);

        let committable_blocks = payload_and_output_list
            .into_iter()
            .map(|payload_and_output| {
                CommittableBlock::new(
                    payload_and_output
                        .0
                        .into_iter()
                        .map(Transaction::UserTransaction)
                        .collect(),
                    payload_and_output.1,
                )
            })
            .collect();

        let commit_future = self
            .executor
            .commit_blocks(committable_blocks, finality_proof);
        async move {
            match commit_future.await {
                Ok(Ok(())) => {
                    counters::BLOCK_COMMIT_DURATION_S
                        .observe_duration(pre_commit_instant.elapsed());
                    if let Err(e) = synchronizer.commit(version).await {
                        error!("failed to notify state synchronizer: {:?}", e);
                    }
                    Ok(())
                }
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e.into()),
            }
        }
            .boxed()
    }

    /// Synchronize to a commit that not present locally.
    fn sync_to(
        &self,
        target: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        counters::STATE_SYNC_COUNT.inc();
        self.synchronizer.sync_to(target).boxed()
    }

    fn committed_trees(&self) -> ExecutedTrees {
        self.executor.committed_trees()
    }

    fn get_epoch_proof(
        &self,
        start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<ValidatorChangeEventWithProof>> + Send>> {
        self.synchronizer.get_epoch_proof(start_epoch).boxed()
    }
}
