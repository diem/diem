// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{
    register_histogram, register_int_counter, register_int_counter_vec, Histogram, IntCounter,
    IntCounterVec,
};
use once_cell::sync::Lazy;

/// Count the number of transactions validated, with a "status" label to
/// distinguish success or failure results.
pub static TRANSACTIONS_VALIDATED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_vm_transactions_validated",
        "Number of transactions validated",
        &["status"]
    )
    .unwrap()
});

/// Count the number of user transactions executed, with a "status" label to
/// distinguish completed vs. discarded transactions.
pub static USER_TRANSACTIONS_EXECUTED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_vm_user_transactions_executed",
        "Number of user transactions executed",
        &["status"]
    )
    .unwrap()
});

/// Count the number of system transactions executed.
pub static SYSTEM_TRANSACTIONS_EXECUTED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_vm_system_transactions_executed",
        "Number of system transactions executed"
    )
    .unwrap()
});

pub static BLOCK_TRANSACTION_COUNT: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "diem_vm_num_txns_per_block",
        "Number of transactions per block"
    )
    .unwrap()
});

pub static TXN_TOTAL_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "diem_vm_txn_total_seconds",
        "Execution time per user transaction"
    )
    .unwrap()
});

pub static TXN_VALIDATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "diem_vm_txn_validation_seconds",
        "Validation time per user transaction"
    )
    .unwrap()
});

pub static TXN_GAS_USAGE: Lazy<Histogram> =
    Lazy::new(|| register_histogram!("diem_vm_txn_gas_usage", "Gas used per transaction").unwrap());

/// Count the number of critical errors. This is not intended for display
/// on a dashboard but rather for triggering alerts.
pub static CRITICAL_ERRORS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("diem_vm_critical_errors", "Number of critical errors").unwrap()
});
