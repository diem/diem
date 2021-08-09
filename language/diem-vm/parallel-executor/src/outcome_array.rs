// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{Error, Result},
    task::{ExecutionStatus, TransactionOutput},
};
use once_cell::sync::OnceCell;

pub(crate) struct OutcomeArray<T, E> {
    // Hold the execution results for each individual transactions. Each cell should be set exactly
    // once.
    results: Vec<OnceCell<ExecutionStatus<T, Error<E>>>>,
}

impl<T: TransactionOutput, E: Send> OutcomeArray<T, E> {
    pub fn new(len: usize) -> OutcomeArray<T, E> {
        OutcomeArray {
            results: (0..len).map(|_| OnceCell::new()).collect(),
        }
    }

    pub fn set_result(&self, idx: usize, res: ExecutionStatus<T, Error<E>>) {
        // We don't need to worry about double writes due to the unique assignment of txn ids within
        // a block here. And each txn id will be scheduled to execute exactly once.

        let entry = &self.results[idx];
        assert!(entry.set(res).is_ok());
    }

    pub fn get_all_results(self, stop_at: usize) -> Result<Vec<T>, E> {
        let len = self.results.len();
        let mut final_results = Vec::with_capacity(stop_at);
        for (idx, status) in self.results.into_iter().take(stop_at).enumerate() {
            let t = match status.into_inner() {
                Some(ExecutionStatus::Success(t)) => t,
                Some(ExecutionStatus::SkipRest(t)) if idx == stop_at - 1 => t,
                Some(ExecutionStatus::SkipRest(_)) => return Err(Error::InvariantViolation),
                Some(ExecutionStatus::Abort(err)) => return Err(err),
                Some(ExecutionStatus::Retry(_)) => return Err(Error::InvariantViolation),
                None => return Err(Error::InvariantViolation),
            };
            final_results.push(t)
        }
        assert!(final_results.len() == stop_at);
        final_results.resize_with(len, T::skip_output);
        Ok(final_results)
    }
}
