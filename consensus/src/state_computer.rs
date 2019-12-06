// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, state_replication::StateComputer};
use anyhow::Result;
use consensus_types::block::Block;
use consensus_types::executed_block::ExecutedBlock;
use executor::{CommittableBlock, ExecutedTrees, Executor, ProcessedVMOutput};
use futures::{Future, FutureExt};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::block_metadata::BlockMetadata;
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
        // TODO: figure out error handling for the prologue txn
        let execute_future = self.executor.execute_block(
            Self::transactions_from_block(block),
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

    /// Compute by Id
    fn compute_by_hash(
        &self,
        // The id of a grandpa block
        grandpa_block_id: &HashValue,
        // The id of a parent block
        parent_block_id: &HashValue,
        // The id of a current block.
        block_id: &HashValue,
        // Transactions to execute.
        transactions: Vec<(BlockMetadata, Self::Payload)>,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedVMOutput>> + Send>> {
        let pre_execution_instant = Instant::now();

        let mut txn_vec = vec![];
        for meta_data_and_txns in transactions {
            txn_vec.push(Transaction::BlockMetadata(meta_data_and_txns.0));
            txn_vec.extend(
                meta_data_and_txns
                    .1
                    .iter()
                    .map(|txn| Transaction::UserTransaction(txn.clone())),
            );
        }

        let execute_future = self.executor.execute_block_by_id(
            txn_vec,
            grandpa_block_id.clone(),
            parent_block_id.clone(),
            block_id.clone(),
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
        blocks: Vec<&ExecutedBlock<Self::Payload>>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        let version = finality_proof.ledger_info().version();
        counters::LAST_COMMITTED_VERSION.set(version as i64);

        let pre_commit_instant = Instant::now();
        let synchronizer = Arc::clone(&self.synchronizer);

        let committable_blocks = blocks
            .into_iter()
            .map(|executed_block| {
                CommittableBlock::new(
                    Self::transactions_from_block(executed_block.block()),
                    Arc::clone(executed_block.output()),
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
                    if let Err(e) = synchronizer.commit().await {
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

    //    fn commit_with_meta_data(
    //        &self,
    //        meta_data_txn: &BlockMetadata,
    //        block: (Self::Payload, Arc<ProcessedVMOutput>),
    //        finality_proof: LedgerInfoWithSignatures,
    //    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
    //        let version = finality_proof.ledger_info().version();
    //        counters::LAST_COMMITTED_VERSION.set(version as i64);
    //
    //        let pre_commit_instant = Instant::now();
    //        let synchronizer = Arc::clone(&self.synchronizer);
    //
    //        let mut txn_vec = vec![Transaction::BlockMetadata(meta_data_txn.clone())];
    //        txn_vec.extend(
    //            block
    //                .0
    //                .iter()
    //                .map(|txn| Transaction::UserTransaction(txn.clone())),
    //        );
    //        let committable_blocks = vec![CommittableBlock::new(txn_vec, block.1)];
    //
    //        let commit_future = self
    //            .executor
    //            .commit_blocks(committable_blocks, finality_proof);
    //        async move {
    //            match commit_future.await {
    //                Ok(Ok(())) => {
    //                    counters::BLOCK_COMMIT_DURATION_S
    //                        .observe_duration(pre_commit_instant.elapsed());
    //                    if let Err(e) = synchronizer.commit().await {
    //                        error!("failed to notify state synchronizer: {:?}", e);
    //                    }
    //                    Ok(())
    //                }
    //                Ok(Err(e)) => Err(e),
    //                Err(e) => Err(e.into()),
    //            }
    //        }
    //            .boxed()
    //    }
    //
    //    fn rollback(&self, block_id: HashValue) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
    //        let pre_rollback_instant = Instant::now();
    //        let rollback_future = self.executor.rollback_by_block_id(block_id);
    //        async move {
    //            match rollback_future.await {
    //                Ok(Ok(())) => {
    //                    counters::BLOCK_COMMIT_DURATION_S
    //                        .observe_duration(pre_rollback_instant.elapsed());
    //                    Ok(())
    //                }
    //                Ok(Err(e)) => Err(e),
    //                Err(e) => Err(e.into()),
    //            }
    //        }
    //            .boxed()
    //    }
    //
    //    fn process_vm_outputs_to_commit(
    //        transactions: (BlockMetadata, Self::Payload),
    //        output: Arc<ProcessedVMOutput>,
    //        parent_trees: &ExecutedTrees) -> Result<ProcessedVMOutput> {
    //        let mut txn_vec = vec![Transaction::BlockMetadata(transactions.0.clone())];
    //        txn_vec.extend(
    //            transactions
    //                .1
    //                .iter()
    //                .map(|txn| Transaction::UserTransaction(txn.clone())),
    //        );
    //
    //        let len = txn_vec.len();
    //        let mut txn_data_list = vec![];
    //        let total_len = output.transaction_data().len();
    //
    //        for i in 0..len {
    //            txn_data_list[i] = output.transaction_data().get((total_len - len - i)).expect("transaction_data is none.").clone();
    //        }
    //
    //        txn_data_list.reverse();
    //
    //        let vm_out_put =  ProcessedVMOutput::new(
    //            txn_data_list,
    //                                                 output.executed_trees().clone(),
    //            output.validators().clone());
    //
    //        Executor::process_vm_outputs(txn_vec, vm_out_put, parent_trees)
    //    }

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
