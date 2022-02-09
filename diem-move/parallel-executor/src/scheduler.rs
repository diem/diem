// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crossbeam::utils::CachePadded;
use crossbeam_queue::SegQueue;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

// (txn_idx, incarnation)
pub type Version = (usize, usize);

pub struct Scheduler {
    // The index of the next txn from the block to be executed for the first time.
    execution_idx: AtomicUsize,
    // Shared validation index. Tracks the next index to validate, resets (is decreased) upon aborts
    // and after execution of a txn that writes outside of its previous incarnation write set.
    validation_idx: AtomicUsize,
    // The the number of times validation_idx was decreased.
    validation_decrease_cnt: AtomicUsize,
    // Number of tasks used to track when transactions can be committed.
    // Initialized with block size and increased and decreased as new validation or execution tasks
    // are created and completed.
    num_active_tasks: AtomicUsize,
    // Shared marker that's set when a thread detects all txns can be committed - so
    // other threads can immediately know without expensive checks.
    done_marker: AtomicUsize,

    // Shared marker that is set as an optimization when all txns have been executed at least once.
    execution_marker_done: AtomicUsize,

    // Shared number of txns to execute: updated before executing a block or when an error or
    // reconfiguration leads to early stopping (at that transaction idx).
    stop_idx: AtomicUsize,
    block_size: usize,

    // shared queue of list of transaction indices that are ready to be executed (e.g.
    // dependency-resolved transactions).
    txn_buffer: SegQueue<usize>,
    txn_dependency: Vec<Arc<Mutex<Vec<usize>>>>, // idx -> txn indices that depend on it.

    incarnation_cnts: Vec<CachePadded<AtomicUsize>>,
    abort_cnts: Vec<CachePadded<AtomicUsize>>,
}

impl Scheduler {
    pub fn new(num_txns: usize) -> Self {
        Self {
            execution_idx: AtomicUsize::new(0),
            validation_idx: AtomicUsize::new(0),
            validation_decrease_cnt: AtomicUsize::new(0),
            num_active_tasks: AtomicUsize::new(num_txns),
            done_marker: AtomicUsize::new(0),
            execution_marker_done: AtomicUsize::new(0),
            stop_idx: AtomicUsize::new(num_txns),
            block_size: num_txns,
            txn_buffer: SegQueue::new(),
            txn_dependency: (0..num_txns)
                .map(|_| Arc::new(Mutex::new(Vec::new())))
                .collect(),
            incarnation_cnts: (0..num_txns)
                .map(|_| CachePadded::new(AtomicUsize::new(0)))
                .collect(),
            abort_cnts: (0..num_txns)
                .map(|_| CachePadded::new(AtomicUsize::new(0)))
                .collect(),
        }
    }

    // Decreases the validation index and increases the corresponding counter.
    fn decrease_validation_idx(&self, target_idx: usize) {
        self.validation_idx.fetch_min(target_idx, Ordering::SeqCst);
        self.validation_decrease_cnt.fetch_add(1, Ordering::SeqCst);
    }

    pub fn abort(&self, txn_idx: usize, incarnation: usize) -> bool {
        // Each abort happens after a corresponding incarnation, hence successfully
        // aborting means "winning" a CAS to increment the abort_cnts to match
        // the incarnation, and there can only be one abort per incarnation.
        if let Ok(_) = self.abort_cnts[txn_idx].compare_exchange(
            incarnation - 1,
            incarnation,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            // Successfully aborted.
            return true;
        }
        false
    }

    // Return the next version for the thread to validate.
    pub fn next_version_to_validate(&self) -> Option<Version> {
        // Optimization for check-done, to avoid num_tasks going up and down.
        let idx_to_validate = self.validation_idx.load(Ordering::SeqCst);
        let num_txns = self.num_txn_to_execute();

        if idx_to_validate >= num_txns {
            self.check_done(num_txns);
            return None;
        }

        loop {
            // Optimization to not keep skipping ahead of the first executions.
            if idx_to_validate >= self.execution_idx.load(Ordering::Relaxed) {
                return None;
            }

            // Need to increment this before incrementing validation_idx.
            self.num_active_tasks.fetch_add(1, Ordering::SeqCst);
            let idx_to_validate = self.validation_idx.fetch_add(1, Ordering::SeqCst);

            if idx_to_validate >= num_txns {
                self.num_active_tasks.fetch_sub(1, Ordering::SeqCst);
                return None;
            }

            let (incarnation, executed) = self.incarnation_status(idx_to_validate);
            if executed {
                // Incarnation was executed and not aborted, thus ready for validation.
                return Some((idx_to_validate, incarnation));
            }

            // Awaiting (re-)execution, not ready to validate
            self.num_active_tasks.fetch_sub(1, Ordering::SeqCst);
        }
    }

    // Return the next transaction to execute. Prioritizes previously executed txns over
    // executing new transactions. Returns None if nothing to execute is found.
    pub fn next_version_to_execute(&self) -> Option<Version> {
        if self.txn_buffer.is_empty() {
            if self.execution_marker_done.load(Ordering::Relaxed) == 1 {
                // All txns executed at least once.
                return None;
            }

            // Fetch the first non-executed txn from the original transaction list
            let next_to_execute = self.execution_idx.fetch_add(1, Ordering::Relaxed);
            if next_to_execute < self.num_txn_to_execute() {
                Some((next_to_execute, 0))
            } else if next_to_execute < self.block_size {
                // If stop index changed, subtract num_tasks for discarded transactions
                // (pre-initialized accounting for the initial execution of all txns).
                self.num_active_tasks.fetch_sub(1, Ordering::SeqCst);
                None
            } else {
                // Everything executed at least once - validation will take care of rest.
                self.execution_marker_done.store(1, Ordering::Relaxed);
                None
            }
        } else if let Some(txn_idx) = self.txn_buffer.pop() {
            // Fetch & return a transaction whose dependency got resolved.
            Some((
                txn_idx,
                self.incarnation_cnts[txn_idx].load(Ordering::SeqCst),
            ))
        } else {
            None
        }
    }

    // Returns current incarnation and whether this incarnation is in an 'executed',
    // i.e. not (yet) aborted state.
    fn incarnation_status(&self, txn_idx: usize) -> (usize, bool) {
        let incarnation_cnt = self.incarnation_cnts[txn_idx].load(Ordering::SeqCst);
        let abort_cnt = self.abort_cnts[txn_idx].load(Ordering::SeqCst);
        // abort_cnt is monotonically increasing and it is read after incarnation_cnt.
        // Thus, when the function returns 'executed', it can be linearized.
        (incarnation_cnt, incarnation_cnt > abort_cnt)
    }

    // When txn depends on another txn, adds txn_idx to the dependency list of the other txn.
    // Returns true if successful, or false, meaning dependency got resolved in the meantime.
    pub fn add_dependency(&self, txn_idx: usize, dep_txn_idx: usize) -> bool {
        // Note: Could pre-check that the txn isn't in executed state, but the caller
        // usually has just observed the read dependency (i.e. not executed state).

        // txn_dependency is initialized for all txn_ids, so unwrap() is safe.
        let mut stored_deps = self.txn_dependency[dep_txn_idx].lock().unwrap();
        let (_, executed) = self.incarnation_status(dep_txn_idx);
        if executed {
            // Current status of dep_txn_idx is 'executed', so the dependency is no longer valid.
            // It is important to return and not add a dependency for liveness. Otherwise, the
            // dependency may never get cleared and the transaction would never execute (zombie).

            return false;
        }

        // Safe to add dependency here (still holding the lock) - a finish_execution with higher
        // incarnation is guaranteed to acquire the same lock later and clear the dependency.
        stored_deps.push(txn_idx);
        true
    }

    // After txn is executed, add its dependencies to the shared buffer for execution.
    // Return Some(version) if only version needs re-validation, otherwise (when
    // revalidate_suffix is true), schedule all higher txns for validation and return None.
    pub fn finish_execution(
        &self,
        version: Version,
        revalidate_suffix: bool,
    ) -> Option<(usize, usize)> {
        // Stores are safe because there is at most one execution at a time.
        let (txn_idx, incarnation) = version;
        self.incarnation_cnts[txn_idx].store(incarnation + 1, Ordering::SeqCst);

        let mut txn_deps: Vec<usize> = {
            let mut stored_deps = self.txn_dependency[txn_idx].lock().unwrap();
            // Holding the lock, take dependency vector.
            std::mem::take(&mut stored_deps)
        };

        // Executing lower txns first is always desirable.
        txn_deps.sort_unstable();
        for dep in txn_deps {
            // Add dependency-resolved txn to the queue of ready to be executed txns.
            self.txn_buffer.push(dep);
        }

        if revalidate_suffix {
            // The transaction execution did smt that requires revalidating everything.
            self.decrease_validation_idx(txn_idx);
            self.num_active_tasks.fetch_sub(1, Ordering::SeqCst);
            None
        } else {
            // Only the transaction itself needs to be revalidated. No need to decrease
            // num_active_tasks here as one correponding (re-) validation task is created.

            Some((txn_idx, incarnation + 1))
        }
    }

    // Validation of txn_idx finished, and whether it's aborted.
    pub fn finish_validation(&self, txn_idx: usize, aborted: bool) {
        if aborted {
            // Schedule strictly higher txns for validation
            // (txn_idx needs to be executed first).
            self.decrease_validation_idx(txn_idx + 1);

            // No need to decrease num_active_tasks here because every abort generates
            // one corresponding (re-)execution task.
        } else {
            self.num_active_tasks.fetch_sub(1, Ordering::SeqCst);
        }
    }

    // Schedule a txn for execution via shared queue.
    pub fn schedule_txn(&self, txn_idx: usize, buffer_threshold: usize) -> bool {
        if self.txn_buffer.len() > buffer_threshold {
            false
        } else {
            self.txn_buffer.push(txn_idx);
            true
        }
    }

    // Reset txn_idx to end the execution earlier. The executor will stop at the smallest
    // `stop_idx` when there are multiple concurrent invocation.
    pub fn set_stop_idx(&self, stop_idx: usize) {
        self.stop_idx.fetch_min(stop_idx, Ordering::Relaxed);
    }

    pub fn num_txn_to_execute(&self) -> usize {
        self.stop_idx.load(Ordering::Relaxed)
    }

    // A lazy, check of whether the scheduler execution is completed.
    // Updates the 'done_marker' so other threads can know by calling done() function below.
    //
    // 1. After the STM execution has completed, validation_idx >= num_txn, num_active_tasks == 0,
    // and validation_decrease_cnt does not change - so it will be successfully detected.
    // 2. If the marker is set, validation_idx >= num_txn && num_active_tasks == 0 must hold
    // at the same time, which implies completion.
    // Proof: O.w. validation_idx must decrease from when it is read to be >= num_txns
    // to when num_active_tasks is read to be 0, but decreasing thread is performing an active tasks,
    // so it must also perform the next instruction in decrease_validation_idx function
    // (validation_decrease_cnt++) before it decreases num_active_tasks so it can become 0.
    // Final check will then detect a change in dec_cnt and not allow a false positive.
    fn check_done(&self, num_txns: usize) {
        let dec_cnt = self.validation_decrease_cnt.load(Ordering::SeqCst);
        if self.validation_idx.load(Ordering::SeqCst) < num_txns
            || self.num_active_tasks.load(Ordering::SeqCst) > 0
        {
            // There are txns to validate.
            return;
        }

        // Re-read and make sure it hasn't changed. If so, everything can be committed
        // and set the done flag.
        if dec_cnt == self.validation_decrease_cnt.load(Ordering::SeqCst) {
            self.done_marker.store(1, Ordering::Release);
        }
    }

    // Checks whether the done marker is set. The marker can only be set by 'check_done'.
    pub fn done(&self) -> bool {
        self.done_marker.load(Ordering::Acquire) == 1
    }
}
