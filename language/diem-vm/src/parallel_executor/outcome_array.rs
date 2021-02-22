// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    transaction::{TransactionOutput, TransactionStatus},
    write_set::WriteSet,
};
use move_core_types::vm_status::VMStatus;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicUsize, Ordering};

unsafe impl Send for OutcomeArray {}
unsafe impl Sync for OutcomeArray {}

pub(crate) struct OutcomeArray {
    results: Vec<OnceCell<(VMStatus, TransactionOutput)>>,

    success_num: AtomicUsize,
    failure_num: AtomicUsize,
}

impl OutcomeArray {
    pub fn new(len: usize) -> OutcomeArray {
        OutcomeArray {
            results: (0..len).map(|_| OnceCell::new()).collect(),

            success_num: AtomicUsize::new(0),
            failure_num: AtomicUsize::new(0),
        }
    }

    pub fn set_result(&self, idx: usize, res: (VMStatus, TransactionOutput), success: bool) {
        // Only one thread can write at the time, so just set it.

        let entry = &self.results[idx];
        entry.set(res).unwrap();

        // #[cfg(test)]
        {
            if success {
                self.success_num.fetch_add(1, Ordering::Relaxed);
            } else {
                self.failure_num.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn get_stats(&self) -> (usize, usize) {
        return (
            self.success_num.load(Ordering::Relaxed),
            self.failure_num.load(Ordering::Relaxed),
        );
    }

    pub fn get_all_results(
        self,
        valid_length: usize,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let mut results = self.results.into_iter()
            .map(|cell| cell.into_inner())
            .collect::<Option<Vec<_>>>()
            .unwrap();

        let _ = results.split_off(valid_length);
        Ok(results)
    }
}
