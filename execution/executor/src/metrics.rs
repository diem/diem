// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{register_histogram, register_int_counter, Histogram, IntCounter};
use once_cell::sync::Lazy;

pub static DIEM_EXECUTOR_EXECUTE_AND_COMMIT_CHUNK_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "diem_executor_execute_and_commit_chunk_seconds",
        // metric description
        "The time spent in seconds of chunk execution and committing in Diem executor"
    )
    .unwrap()
});

pub static DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "diem_executor_vm_execute_block_seconds",
        // metric description
        "The time spent in seconds of vm block execution in Diem executor"
    )
    .unwrap()
});

pub static DIEM_EXECUTOR_ERRORS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("diem_executor_error_total", "Cumulative number of errors").unwrap()
});

pub static DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "diem_executor_execute_block_seconds",
        // metric description
        "The total time spent in seconds of block execution in Diem executor "
    )
    .unwrap()
});

pub static DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "diem_executor_commit_blocks_seconds",
        // metric description
        "The total time spent in seconds of commiting blocks in Diem executor "
    )
    .unwrap()
});

pub static DIEM_EXECUTOR_SAVE_TRANSACTIONS_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "diem_executor_save_transactions_seconds",
        // metric description
        "The time spent in seconds of calling save_transactions to storage in Diem executor"
    )
    .unwrap()
});

pub static DIEM_EXECUTOR_TRANSACTIONS_SAVED: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "diem_executor_transactions_saved",
        // metric description
        "The number of transactions saved to storage in Diem executor"
    )
    .unwrap()
});
