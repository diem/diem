// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crossbeam_queue::SegQueue;
use mvhashmap::Version;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

#[repr(usize)]
enum ExecutionStatus {
    Executed = 1,
    NotExecuted = 0,
}

pub struct Scheduler {
    // Shared index (version) of the next txn to be executed from the original transaction sequence.
    execution_marker: AtomicUsize,
    // Shared number of txns to execute: updated before executing a block or when an error or
    // reconfiguration leads to early stopping (at that transaction version).
    stop_at_version: AtomicUsize,

    txn_buffer: SegQueue<usize>, // shared queue of list of dependency-resolved transactions.
    // TODO: Do we need padding here?
    txn_dependency: Vec<Arc<RwLock<Vec<usize>>>>, // version -> txns that depend on it.
    txn_status: Vec<AtomicUsize>,                 // version -> execution status.
}

impl Scheduler {
    pub fn new(num_txns: usize) -> Self {
        Self {
            execution_marker: AtomicUsize::new(0),
            stop_at_version: AtomicUsize::new(num_txns),
            txn_buffer: SegQueue::new(),
            txn_dependency: (0..num_txns)
                .map(|_| Arc::new(RwLock::new(Vec::new())))
                .collect(),
            txn_status: (0..num_txns)
                .map(|_| AtomicUsize::new(ExecutionStatus::NotExecuted as usize))
                .collect(),
        }
    }

    // Return the next txn id for the thread to execute: first fetch from the shared queue that
    // stores dependency-resolved txns, then fetch from the original ordered txn sequence.
    // Return Some(id) if found the next transaction, else return None.
    pub fn next_txn_to_execute(&self) -> Option<Version> {
        // Fetch txn from txn_buffer
        match self.txn_buffer.pop() {
            Some(version) => Some(version),
            None => {
                // Fetch the first non-executed txn from the original transaction list
                let next_to_execute = self.execution_marker.fetch_add(1, Ordering::Relaxed);
                if next_to_execute < self.num_txn_to_execute() {
                    Some(next_to_execute)
                } else {
                    // Everything executed at least once - validation will take care of rest.
                    None
                }
            }
        }
    }

    // Invoked when txn depends on another txn, adds version to the dependency list the other txn.
    // Return true if successful, otherwise dependency resolved in the meantime, return false.
    pub fn add_dependency(&self, version: Version, dep_version: Version) -> bool {
        // Could pre-check that the txn isn't in executed state, but shouldn't matter much since
        // the caller usually has just observed the read dependency (so not executed state).

        // txn_dependency is initialized for all versions, so unwrap() is safe.
        let mut stored_deps = self.txn_dependency[dep_version].write().unwrap();
        if self.txn_status[dep_version].load(Ordering::Acquire)
            != ExecutionStatus::Executed as usize
        {
            stored_deps.push(version);
            return true;
        }
        false
    }

    // After txn is executed, add its dependencies to the shared buffer for execution.
    pub fn finish_execution(&self, version: Version) {
        self.txn_status[version].store(ExecutionStatus::Executed as usize, Ordering::Release);
        let mut version_deps: Vec<usize> = {
            // we want to make things fast inside the lock, so use take instead of clone
            let mut stored_deps = self.txn_dependency[version].write().unwrap();
            std::mem::take(&mut stored_deps)
        };

        version_deps.sort_unstable();
        for dep in version_deps {
            self.txn_buffer.push(dep);
        }
    }

    // Reset the txn version/id to end execution earlier. The executor will stop at the smallest
    // `stop_version` when there are multiple concurrent invocation.
    pub fn set_stop_version(&self, stop_version: Version) {
        self.stop_at_version
            .fetch_min(stop_version, Ordering::Relaxed);
    }

    // Adding version to the ready queue.
    pub fn add_transaction(&self, version: Version) {
        self.txn_buffer.push(version)
    }

    // Get the last txn version/id
    pub fn num_txn_to_execute(&self) -> Version {
        self.stop_at_version.load(Ordering::Relaxed)
    }
}
