// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    outcome_array::OutcomeArray,
    scheduler::Scheduler,
    task::{ExecutionStatus, ExecutorTask, ReadWriteSetInferencer, Transaction, TransactionOutput},
};
use anyhow::{bail, Result as AResult};
use mvhashmap::{MVHashMap, Version};
use num_cpus;
use rayon::{prelude::*, scope};
use std::{
    cmp::{max, min},
    hash::Hash,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use diem_logger::prelude::*;

pub struct MVHashMapView<'a, K, V> {
    map: &'a MVHashMap<K, V>,
    version: Version,
    scheduler: &'a Scheduler,
    has_unexpected_read: AtomicBool,
}

impl<'a, K: Hash + Clone + Eq, V> MVHashMapView<'a, K, V> {
    pub fn read(&self, key: &K) -> AResult<Option<&V>> {
        match self.map.read(key, self.version) {
            Ok(v) => Ok(Some(v)),
            Err(None) => Ok(None),
            Err(Some(dep_idx)) => {
                // Don't start execution transaction `self.version` until `dep_idx` is computed.
                if !self.scheduler.add_dependency(self.version, dep_idx) {
                    // dep_idx is already executed, push `self.version` to ready queue.
                    self.scheduler.add_transaction(self.version);
                }
                self.has_unexpected_read.fetch_or(true, Ordering::Relaxed);
                bail!("Read dependency is not computed, retry later")
            }
        }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn has_unexpected_read(&self) -> bool {
        self.has_unexpected_read.load(Ordering::Relaxed)
    }
}

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
        if signature_verified_block.is_empty() {
            return Ok(vec![]);
        }
        let num_txns = signature_verified_block.len();
        let chunks_size = max(1, num_txns / self.num_cpus);

        let log = num_txns > 1000;
        let mut now = std::time::Instant::now();

        // Get the read and write dependency for each transaction.
        let infer_result: Vec<_> = {
            match signature_verified_block
                .par_iter()
                .with_min_len(chunks_size)
                .map(|txn| self.inferencer.infer_reads_writes(txn))
                .collect::<AResult<Vec<_>>>()
            {
                Ok(res) => res,
                // Inferencer passed in by user failed to get the read/writeset of a transaction,
                // abort parallel execution.
                Err(_) => return Err(Error::InferencerError),
            }
        };

        if log {
            info!("Time to infer: {:?}", now.elapsed());
            now = std::time::Instant::now();
        }



        // Use write analysis result to construct placeholders.
        let path_version_tuples: Vec<(T::Key, usize)> = infer_result
            .par_iter()
            .enumerate()
            .with_min_len(chunks_size)
            .fold(Vec::new, |mut acc, (idx, accesses)| {
                acc.extend(
                    accesses
                        .keys_written
                        .clone()
                        .into_iter()
                        .map(|ap| (ap, idx)),
                );
                acc
            })
            .flatten()
            .collect();

        let (versioned_data_cache, max_dependency_level) =
            MVHashMap::new_from_parallel(path_version_tuples);

        if max_dependency_level == 0 {
            return Err(Error::InferencerError);
        }

        let outcomes = OutcomeArray::new(num_txns);

        let scheduler = Arc::new(Scheduler::new(num_txns));

        if log {
            info!("Time to start up: {:?}", now.elapsed());
            now = std::time::Instant::now();
        }

        scope(|s| {
            // How many threads to use?
            let compute_cpus = min(1 + (num_txns / 50), self.num_cpus - 1); // Ensure we have at least 50 tx per thread.
            let compute_cpus = min(num_txns / max_dependency_level, compute_cpus); // Ensure we do not higher rate of conflict than concurrency.

            info!("Num txns: {:?}, max_dependency: {:?}, CPUs: {:?}, threads: {:?}", num_txns, max_dependency_level, self.num_cpus, compute_cpus);
            for _ in 0..(compute_cpus) {
                s.spawn(|_| {
                    let scheduler = Arc::clone(&scheduler);
                    // Make a new executor per thread.
                    let task = E::init(task_initial_arguments);

                    while let Some(idx) = scheduler.next_txn_to_execute() {
                        let txn = &signature_verified_block[idx];
                        let txn_accesses = &infer_result[idx];

                        // If the txn has unresolved dependency, adds the txn to deps_mapping of its dependency (only the first one) and continue
                        if txn_accesses.keys_read.iter().any(|k| {
                            match versioned_data_cache.read(k, idx) {
                                Err(Some(dep_id)) => scheduler.add_dependency(idx, dep_id),
                                Ok(_) | Err(None) => false,
                            }
                        }) {
                            // This causes a PAUSE on an x64 arch, and takes 140 cycles. Allows other
                            // core to take resources and better HT.
                            ::std::hint::spin_loop();
                            continue;
                        }

                        // Process the output of a transaction
                        let view = MVHashMapView {
                            map: &versioned_data_cache,
                            version: idx,
                            scheduler: &scheduler,
                            has_unexpected_read: AtomicBool::new(false),
                        };
                        let execute_result = task.execute_transaction(&view, txn);
                        if view.has_unexpected_read() {
                            // We've already added this transaction back to the scheduler in the
                            // MVHashmapView where this bit is set, thus it is safe to continue
                            // here.
                            continue;
                        }
                        let commit_result =
                            match execute_result {
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
                            };

                        for write in txn_accesses.keys_written.iter() {
                            // Unwrap here is fine because all writes here should be in the mvhashmap.
                            assert!(versioned_data_cache.skip_if_not_set(write, idx).is_ok());
                        }

                        scheduler.finish_execution(idx);
                        outcomes.set_result(idx, commit_result);
                    }
                });
            }
        });

        if log {
            info!("Time to compute: {:?}", now.elapsed());
            now = std::time::Instant::now();
        }


        // Splits the head of the vec of results that are valid
        let valid_results_length = scheduler.num_txn_to_execute();

        // Dropping large structures is expensive -- do this is a separate thread.
        ::std::thread::spawn(move || {
            drop(scheduler);
            drop(infer_result);
            drop(signature_verified_block); // Explicit drops to measure their cost.
            drop(versioned_data_cache);
        });

        let results = outcomes.get_all_results(valid_results_length);

        if log {
            info!("Time to cleanup: {:?}", now.elapsed());
            now = std::time::Instant::now();
        }

        results
    }
}
