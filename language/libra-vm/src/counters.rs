// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{
    register_histogram, register_int_counter_vec, register_int_gauge, Histogram, IntCounterVec,
    IntGauge,
};
use once_cell::sync::Lazy;

/// Count the number of transactions verified, with a "status" label to
/// distinguish success or failure results.
pub static TRANSACTIONS_VERIFIED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_vm_transactions_verified",
        "Number of transactions verified",
        &["status"]
    )
    .unwrap()
});

/// Count the number of transactions executed, with a "status" label to
/// distinguish completed vs. discarded transactions.
pub static TRANSACTIONS_EXECUTED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_vm_transactions_executed",
        "Number of transactions executed",
        &["status"]
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

pub static TXN_VERIFICATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_verification_seconds",
        "Histogram of verification time per transaction"
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

pub static TXN_EXECUTION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_execution_seconds",
        "Histogram of execution time per transaction"
    )
    .unwrap()
});

pub static TXN_PROLOGUE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_prologue_seconds",
        "Histogram of prologue time per transaction"
    )
    .unwrap()
});

pub static TXN_EPILOGUE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_epilogue_seconds",
        "Histogram of epilogue time per transaction"
    )
    .unwrap()
});

pub static TXN_EXECUTION_GAS_USAGE: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_execution_gas_usage",
        "Histogram for the gas used during txn execution"
    )
    .unwrap()
});

pub static TXN_TOTAL_GAS_USAGE: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_vm_txn_total_gas_usage",
        "Histogram for the total gas used for txns"
    )
    .unwrap()
});
