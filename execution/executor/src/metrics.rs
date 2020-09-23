// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{register_histogram, register_int_counter, Histogram, IntCounter};
use once_cell::sync::Lazy;

pub static LIBRA_EXECUTOR_VM_EXECUTE_CHUNK_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_executor_vm_execute_chunk_seconds",
        // metric description
        "The time spent in seconds of vm chunk execution in Libra executor"
    )
    .unwrap()
});

pub static LIBRA_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_executor_vm_execute_block_seconds",
        // metric description
        "The time spent in seconds of vm block execution in Libra executor"
    )
    .unwrap()
});

pub static LIBRA_EXECUTOR_ERRORS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_executor_error_total", "Cumulative number of errors").unwrap()
});

pub static LIBRA_EXECUTOR_EXECUTE_BLOCK_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_executor_execute_block_seconds",
        // metric description
        "The total time spent in seconds of block execution in Libra executor "
    )
    .unwrap()
});

pub static LIBRA_EXECUTOR_COMMIT_BLOCKS_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_executor_commit_blocks_seconds",
        // metric description
        "The total time spent in seconds of commiting blocks in Libra executor "
    )
    .unwrap()
});

pub static LIBRA_EXECUTOR_SAVE_TRANSACTIONS_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_executor_save_transactions_seconds",
        // metric description
        "The time spent in seconds of calling save_transactions to storage in Libra executor"
    )
    .unwrap()
});

pub static LIBRA_EXECUTOR_TRANSACTIONS_SAVED: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_executor_transactions_saved",
        // metric description
        "The number of transactions saved to storage in Libra executor"
    )
    .unwrap()
});
