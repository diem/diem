// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    errors::*,
    outcome_array::OutcomeArray,
    scheduler::Scheduler,
    task::{ExecutionStatus, ExecutorTask, Transaction, TransactionOutput},
    txn_last_input_output::{ReadDescriptor, TxnLastInputOutput},
};
use anyhow::{bail, Result as AResult};
use mvhashmap::MVHashMap;
use num_cpus;
use rayon::scope;
use std::{
    collections::HashSet,
    hash::Hash,
    hint,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

const NOTHING_ITER_THRESHOLD: usize = 10000;

pub struct DynamicWorkers<'a> {
    thread_idx: usize,
    // Number of consecutive times thread tried and failed to grab a validation task.
    nothing_to_validate_cnt: usize,
    // Number of consecutive times thread tried and failed to grab an execution task.
    nothing_to_execute_cnt: usize,

    // Threads 0..num_validator_workers prioritize validation.
    num_validator_workers: &'a AtomicUsize,
    // Maximum number of threads that prioritize validation.
    max_validator_workers: usize,
}

impl<'a> DynamicWorkers<'a> {
    pub fn new(
        thread_id: usize,
        num_validator_workers: &'a AtomicUsize,
        max_validator_workers: usize,
    ) -> Self {
        Self {
            thread_idx: thread_id,
            nothing_to_validate_cnt: 0,
            nothing_to_execute_cnt: 0,
            num_validator_workers,
            max_validator_workers,
        }
    }

    pub fn validator(&self) -> bool {
        self.thread_idx < self.num_validator_workers.load(Ordering::Relaxed)
    }

    pub fn validated(&mut self) {
        self.nothing_to_validate_cnt = 0;
    }

    pub fn executed(&mut self) {
        self.nothing_to_execute_cnt = 0;
    }

    // Returns true if the thread can hint::spin(), since it could grab no validation task.
    pub fn nothing_to_validate(&mut self) -> bool {
        // Don't let 0 spin, may need to set done.
        if self.thread_idx == 0 {
            return false;
        }
        if self.thread_idx == self.num_validator_workers.load(Ordering::Relaxed) - 1 {
            self.nothing_to_validate_cnt += 1;
            // TODO: a config parameter
            if self.nothing_to_validate_cnt == NOTHING_ITER_THRESHOLD {
                self.num_validator_workers.fetch_sub(1, Ordering::Relaxed);
                self.nothing_to_validate_cnt = 0;
                return false;
            }
        }
        return true;
    }

    // Returns true if the thread can hint::spin(), since it could grab no execution task.
    pub fn nothing_to_execute(&mut self) -> bool {
        if self.thread_idx >= self.max_validator_workers {
            return false;
        }
        if self.thread_idx < self.max_validator_workers
            && self.thread_idx == self.num_validator_workers.load(Ordering::Relaxed)
        {
            self.nothing_to_execute_cnt += 1;
            // TODO: config parameter
            if self.nothing_to_execute_cnt == NOTHING_ITER_THRESHOLD {
                self.num_validator_workers.fetch_add(1, Ordering::Relaxed);
                self.nothing_to_execute_cnt = 0;
                return false;
            }
        }
        return true;
    }
}

pub struct MVHashMapView<'a, K, V> {
    versioned_map: &'a MVHashMap<K, V>,
    txn_idx: usize,
    scheduler: &'a Scheduler,
    read_dependency: AtomicBool,
    captured_reads: Arc<Mutex<Vec<ReadDescriptor<K>>>>,
}

impl<'a, K: PartialOrd + Send + Clone + Hash + Eq, V: Clone + Send + Sync> MVHashMapView<'a, K, V> {
    // Drains the captured reads.
    pub fn take_reads(&self) -> Vec<ReadDescriptor<K>> {
        let mut reads = self.captured_reads.lock().unwrap();
        std::mem::take(&mut reads)
    }

    // Captures a read from the VM execution.
    pub fn read(&self, key: &K) -> AResult<Option<V>> {
        loop {
            match self.versioned_map.read(key, self.txn_idx) {
                Ok((v, txn_idx, incarnation)) => {
                    self.captured_reads
                        .lock()
                        .unwrap()
                        .push(ReadDescriptor::new(
                            key.clone(),
                            Some((txn_idx, incarnation)),
                        ));
                    return Ok(v);
                }
                Err(None) => {
                    self.captured_reads
                        .lock()
                        .unwrap()
                        .push(ReadDescriptor::new(key.clone(), None));
                    return Ok(None);
                }
                Err(Some((dep_idx, _exec_id))) => {
                    // Don't start execution transaction `self.txn_idx` until `dep_idx` is computed.
                    if self.scheduler.add_dependency(self.txn_idx, dep_idx) {
                        // dep_idx is already executed, push `self.txn_idx` to ready queue.
                        self.read_dependency.store(true, Ordering::Relaxed);
                        bail!("Read dependency is not computed, retry later")
                    }
                }
            };
        }
    }

    pub fn txn_idx(&self) -> usize {
        self.txn_idx
    }

    pub fn read_dependency(&self) -> bool {
        self.read_dependency.load(Ordering::Relaxed)
    }
}

pub struct ParallelTransactionExecutor<T: Transaction, E: ExecutorTask> {
    num_cpus: usize,
    phantom: PhantomData<(T, E)>,
}

impl<T, E> ParallelTransactionExecutor<T, E>
    where
        T: Transaction,
        E: ExecutorTask<T = T>,
{
    pub fn new() -> Self {
        Self {
            num_cpus: num_cpus::get(),
            phantom: PhantomData,
        }
    }

    pub fn execute(
        &self,
        version_to_execute: Option<(usize, usize)>,
        signature_verified_block: &Vec<T>,
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &Scheduler,
        task: &E,
    ) -> Option<(usize, usize)> {
        let (txn_to_execute, incarnation) = version_to_execute.unwrap();
        let txn = &signature_verified_block[txn_to_execute];

        // TODO: does it help or hurt perf?
        // TODO: get rid of it once we have async interface
        if incarnation > 0
            && last_input_output
            .read_set(txn_to_execute)
            .iter()
            .any(
                |r| match versioned_data_cache.read(r.path(), txn_to_execute) {
                    Err(Some((dep_id, _))) => scheduler.add_dependency(txn_to_execute, dep_id),
                    Ok(_) | Err(None) => false,
                },
            )
        {
            //The transaction has a read dependency. Was not executed and thus there is nothing to validate.
            return None;
        }

        let state_view = MVHashMapView {
            versioned_map: &versioned_data_cache,
            txn_idx: txn_to_execute,
            scheduler: &scheduler,
            read_dependency: AtomicBool::new(false),
            captured_reads: Arc::new(Mutex::new(Vec::new())),
        };

        // VM execution.
        let execute_result = task.execute_transaction(&state_view, txn);

        if state_view.read_dependency() {
            // Encountered and already handled (added to Scheduler) a read dependency.
            return None;
        }

        let mut prev_write_set: HashSet<T::Key> = last_input_output.write_set(txn_to_execute);

        // For tracking whether the recent execution wrote outside of the previous write set.
        let mut writes_outside = false;
        let mut apply_writes = |output: &<E as ExecutorTask>::Output| {
            for (k, v) in output.get_writes().into_iter() {
                if !prev_write_set.remove(&k) {
                    writes_outside = true
                }
                versioned_data_cache.write(&k, txn_to_execute, incarnation, Some(v));
            }
        };

        let commit_result = match execute_result {
            ExecutionStatus::Success(output) => {
                // Commit the side effects to the versioned_data_cache.
                apply_writes(&output);
                ExecutionStatus::Success(output)
            }
            ExecutionStatus::SkipRest(output) => {
                // Commit and skip the rest of the transactions.
                apply_writes(&output);
                scheduler.set_stop_idx(txn_to_execute + 1);
                ExecutionStatus::SkipRest(output)
            }
            ExecutionStatus::Abort(err) => {
                // Abort the execution with user defined error.
                scheduler.set_stop_idx(txn_to_execute + 1);
                ExecutionStatus::Abort(Error::UserError(err.clone()))
            }
        };

        // Remove entries from previous write set that were not overwritten.
        for k in &prev_write_set {
            versioned_data_cache.delete(k, txn_to_execute);
        }

        last_input_output.record(txn_to_execute, state_view.take_reads(), commit_result);
        scheduler.finish_execution((txn_to_execute, incarnation), writes_outside)
    }

    pub fn validate(
        &self,
        version_to_validate: Option<(usize, usize)>,
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &Scheduler,
    ) -> Option<(usize, usize)> {
        let mut ret = None;

        let (idx_to_validate, incarnation) = version_to_validate.unwrap();
        let read_set = last_input_output.read_set(idx_to_validate);

        let valid = read_set.iter().all(|r| {
            match versioned_data_cache.read(r.path(), idx_to_validate) {
                Ok((_, txn_idx, incarnation_num)) => r.validate(Some((txn_idx, incarnation_num))),
                Err(Some(_)) => false, // Dependency implies a validation failure.
                Err(None) => r.validate(None),
            }
        });

        let aborted = if !valid && scheduler.abort(idx_to_validate, incarnation) {
            // Not valid and successfully aborted.

            // Abort succeeded, mark the latest write-set as estimates.
            for k in &last_input_output.write_set(idx_to_validate) {
                versioned_data_cache.make_estimate(k, idx_to_validate);
            }

            // TODO: investigate performance trade-offs
            if !scheduler.schedule_txn(idx_to_validate, self.num_cpus / 4) {
                ret = Some((idx_to_validate, incarnation));
            }

            true
        } else {
            false
        };

        scheduler.finish_validation(idx_to_validate, aborted);

        ret
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
        let versioned_data_cache = MVHashMap::new();
        let outcomes = OutcomeArray::new(num_txns);
        let compute_cpus = self.num_cpus;
        let last_input_output = TxnLastInputOutput::new(num_txns);
        let scheduler = Scheduler::new(num_txns);
        let thread_idx_generator = AtomicUsize::new(0);
        let num_validator_workers = AtomicUsize::new(compute_cpus / 4);
        let max_validator_workers = compute_cpus / 2;

        scope(|s| {
            dbg!(
                "Launching {} threads to execute... total txns: {:?}",
                compute_cpus,
                scheduler.num_txn_to_execute(),
            );

            for _ in 0..(compute_cpus) {
                s.spawn(|_| {
                    // Make executor for each thread.
                    let task = E::init(task_initial_arguments);

                    let id: usize = thread_idx_generator.fetch_add(1, Ordering::Relaxed);

                    let mut dynamic_workers: DynamicWorkers =
                        DynamicWorkers::new(id, &num_validator_workers, max_validator_workers);

                    let mut version_to_validate = None;
                    let mut version_to_execute = None;

                    loop {
                        if scheduler.done() {
                            // The work is done, break from the loop.
                            break;
                        }

                        if dynamic_workers.validator() || version_to_validate.is_some() {
                            if version_to_validate.is_none() {
                                version_to_validate = scheduler.next_version_to_validate();
                            }

                            if version_to_validate.is_some() {
                                version_to_execute = self.validate(
                                    version_to_validate,
                                    &last_input_output,
                                    &versioned_data_cache,
                                    &scheduler,
                                );
                                version_to_validate = None;
                                dynamic_workers.validated();
                            } else {
                                if dynamic_workers.nothing_to_validate() {
                                    hint::spin_loop();
                                }
                            }

                            if version_to_execute.is_none() {
                                // Validation successfully completed or someone already aborted or worked moved to executing threads,
                                // continue to the work loop.
                                continue;
                            }
                        }

                        if version_to_execute.is_none() {
                            version_to_execute = scheduler.next_version_to_execute();
                        }

                        if version_to_execute.is_some() {
                            version_to_validate = self.execute(
                                version_to_execute,
                                &signature_verified_block,
                                &last_input_output,
                                &versioned_data_cache,
                                &scheduler,
                                &task,
                            );
                            version_to_execute = None;
                            dynamic_workers.executed();
                        } else if dynamic_workers.nothing_to_execute() {
                            hint::spin_loop();
                        }
                    }
                });
            }
        });

        // Extract outputs in parallel
        let valid_results_length = scheduler.num_txn_to_execute();
        let commit_index = AtomicUsize::new(0);
        scope(|s| {
            for _ in 0..(compute_cpus) {
                s.spawn(|_| {
                    loop {
                        let next_to_commit = commit_index.fetch_add(1, Ordering::Relaxed);
                        if next_to_commit >= valid_results_length {
                            break;
                        }

                        let commit_result = last_input_output.take_output(next_to_commit);
                        // Updates the outcome array
                        outcomes.set_result(next_to_commit, commit_result);
                    }
                });
            }
        });

        ::std::thread::spawn(move || {
            // Explicit async drops.
            drop(signature_verified_block);
            drop(versioned_data_cache);
            drop(scheduler);
        });
        outcomes.get_all_results(valid_results_length)
    }
}
