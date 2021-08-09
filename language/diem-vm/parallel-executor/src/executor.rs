// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    outcome_array::OutcomeArray,
    scheduler::Scheduler,
    task::{ExecutionStatus, ExecutorTask, ReadWriteSetInferencer, Transaction, TransactionOutput},
};
use anyhow::Result as AResult;
use mvhashmap::MVHashMap;
use num_cpus;
use rayon::{prelude::*, scope};
use std::{
    cmp::{max, min},
    marker::PhantomData,
    sync::Arc,
};

pub struct ParallelTransactionExecutor<T: Transaction, E: ExecutorTask, I: ReadWriteSetInferencer> {
    num_cpus: usize,
    inferencer: I,
    phantom: PhantomData<(T, E, I)>,
}

impl<T, E, I> ParallelTransactionExecutor<T, E, I>
where
    T: Transaction,
    E: ExecutorTask<T = T>,
    I: ReadWriteSetInferencer<T = T>,
{
    pub fn new(inferencer: I) -> Self {
        Self {
            num_cpus: num_cpus::get(),
            inferencer,
            phantom: PhantomData,
        }
    }

    pub fn execute_transactions_parallel(
        &self,
        task_initial_arguments: E::Argument,
        signature_verified_block: Vec<T>,
    ) -> Result<Vec<E::Output>, E::Error> {
        let num_txns = signature_verified_block.len();
        let chunks_size = max(1, num_txns / self.num_cpus);

        // Get the read and write dependency for each transaction.
        let infer_result: Vec<_> = {
            match signature_verified_block
                .par_iter()
                .with_min_len(chunks_size)
                .map(|txn| {
                    Ok((
                        self.inferencer.infer_reads(txn)?,
                        self.inferencer.infer_writes(txn)?,
                    ))
                })
                .collect::<AResult<Vec<_>>>()
            {
                Ok(res) => res,
                // Inferencer passed in by user failed to get the read/writeset of a transaction,
                // abort parallel execution.
                Err(_) => return Err(Error::InferencerError),
            }
        };

        // Use write analysis result to construct placeholders.
        let path_version_tuples: Vec<(T::Key, usize)> = infer_result
            .par_iter()
            .enumerate()
            .with_min_len(chunks_size)
            .fold(Vec::new, |mut acc, (idx, (_, txn_writes))| {
                acc.extend(txn_writes.clone().into_iter().map(|ap| (ap, idx)));
                acc
            })
            .flatten()
            .collect();

        let (versioned_data_cache, max_dependency_level) =
            MVHashMap::new_from_parallel(path_version_tuples);
        let outcomes = OutcomeArray::new(num_txns);

        let scheduler = Arc::new(Scheduler::new(num_txns));

        scope(|s| {
            // How many threads to use?
            let compute_cpus = min(1 + (num_txns / 50), self.num_cpus - 1); // Ensure we have at least 50 tx per thread.
            let compute_cpus = min(num_txns / max_dependency_level, compute_cpus); // Ensure we do not higher rate of conflict than concurrency.

            for _ in 0..(compute_cpus) {
                s.spawn(|_| {
                    let scheduler = Arc::clone(&scheduler);
                    // Make a new executor per thread.
                    let task = E::init(task_initial_arguments);

                    while let Some(idx) = scheduler.next_txn_to_execute() {
                        let txn = &signature_verified_block[idx];
                        let (reads, writes) = &infer_result[idx];

                        // If the txn has unresolved dependency, adds the txn to deps_mapping of its dependency (only the first one) and continue
                        if reads
                            .iter()
                            .any(|k| match versioned_data_cache.read(k, idx) {
                                Err(Some(dep_id)) => scheduler.add_dependency(idx, dep_id),
                                Ok(_) | Err(None) => false,
                            })
                        {
                            // This causes a PAUSE on an x64 arch, and takes 140 cycles. Allows other
                            // core to take resources and better HT.
                            ::std::hint::spin_loop();
                            continue;
                        }

                        // Process the output of a transaction
                        let commit_result =
                            match task.execute_transaction(versioned_data_cache.view(idx), txn) {
                                ExecutionStatus::Success(output) => {
                                    // Commit the side effects to the versioned_data_cache.
                                    if output.get_writes().into_iter().all(|(k, v)| {
                                        versioned_data_cache.write(&k, idx, v).is_ok()
                                    }) {
                                        ExecutionStatus::Success(output)
                                    } else {
                                        // Failed to write to the versioned data cache as
                                        // transaction write to a key that wasn't estimated by the
                                        // inferencer, aborting the entire execution.
                                        ExecutionStatus::Abort(Error::UnestimatedWrite)
                                    }
                                }
                                ExecutionStatus::SkipRest(output) => {
                                    // Commit and skip the rest of the transactions.
                                    if output.get_writes().into_iter().all(|(k, v)| {
                                        versioned_data_cache.write(&k, idx, v).is_ok()
                                    }) {
                                        scheduler.set_stop_version(idx + 1);
                                        ExecutionStatus::SkipRest(output)
                                    } else {
                                        // Failed to write to the versioned data cache as
                                        // transaction write to a key that wasn't estimated by the
                                        // inferencer, aborting the entire execution.
                                        ExecutionStatus::Abort(Error::UnestimatedWrite)
                                    }
                                }
                                ExecutionStatus::Abort(err) => {
                                    // Abort the execution with user defined error.
                                    scheduler.set_stop_version(idx + 1);
                                    ExecutionStatus::Abort(Error::UserError(err.clone()))
                                }
                                ExecutionStatus::Retry(dep_idx) => {
                                    // Mark transaction `idx` to be dependent on `dep_idx`.
                                    if !scheduler.add_dependency(idx, dep_idx) {
                                        // dep_idx is already executed, push idx to ready queue.
                                        scheduler.add_transaction(idx);
                                    }
                                    continue;
                                }
                            };

                        for write in writes.iter() {
                            // Unwrap here is fine because all writes here should be in the mvhashmap.
                            assert!(versioned_data_cache.skip_if_not_set(write, idx).is_ok());
                        }

                        scheduler.finish_execution(idx);
                        outcomes.set_result(idx, commit_result);
                    }
                });
            }
        });

        // Splits the head of the vec of results that are valid
        let valid_results_length = scheduler.num_txn_to_execute();

        // Dropping large structures is expensive -- do this is a separate thread.
        ::std::thread::spawn(move || {
            drop(scheduler);
            drop(infer_result);
            drop(signature_verified_block); // Explicit drops to measure their cost.
            drop(versioned_data_cache);
        });

        outcomes.get_all_results(valid_results_length)
    }
}
