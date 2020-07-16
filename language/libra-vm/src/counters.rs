// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{
    register_histogram, register_int_counter, register_int_counter_vec, register_int_gauge,
    Histogram, IntCounter, IntCounterVec, IntGauge,
};
use once_cell::sync::Lazy;

/// Count the number of transactions validated, with a "status" label to
/// distinguish success or failure results.
pub static TRANSACTIONS_VALIDATED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_vm_transactions_validated",
        "Number of transactions validated",
        &["status"]
    )
    .unwrap()
});

/// Count the number of user transactions executed, with a "status" label to
/// distinguish completed vs. discarded transactions.
pub static USER_TRANSACTIONS_EXECUTED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_vm_user_transactions_executed",
        "Number of user transactions executed",
        &["status"]
    )
    .unwrap()
});

/// Count the number of system transactions executed.
pub static SYSTEM_TRANSACTIONS_EXECUTED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_vm_system_transactions_executed",
        "Number of system transactions executed"
    )
    .unwrap()
});

pub static BLOCK_TRANSACTION_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_vm_block_transaction_count",
        "Number of transaction per block"
    )
    .unwrap()
});

pub static TXN_TOTAL_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_total_seconds",
        "Histogram of total time per transaction"
    )
    .unwrap()
});

pub static TXN_VALIDATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_validation_seconds",
        "Histogram of validation time per transaction"
    )
    .unwrap()
});

pub static TXN_GAS_USAGE: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_gas_usage",
        "Histogram for the gas used for txns"
    )
    .unwrap()
});
