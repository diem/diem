// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{
    register_histogram_vec, register_int_counter, register_int_gauge, register_int_gauge_vec,
    HistogramVec, IntCounter, IntGauge, IntGaugeVec,
};
use once_cell::sync::Lazy;

pub static LIBRA_STORAGE_LEDGER: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "libra_storage_ledger",
        // metric description
        "Libra storage ledger counters",
        // metric labels (dimensions)
        &["type"]
    )
    .unwrap()
});

pub static LIBRA_STORAGE_CF_SIZE_BYTES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "libra_storage_cf_size_bytes",
        // metric description
        "Libra storage Column Family size in bytes",
        // metric labels (dimensions)
        &["cf_name"]
    )
    .unwrap()
});

pub static LIBRA_STORAGE_COMMITTED_TXNS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_storage_committed_txns",
        "Libra storage committed transactions"
    )
    .unwrap()
});

pub static LIBRA_STORAGE_LATEST_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_storage_latest_transaction_version",
        "Libra storage latest transaction version"
    )
    .unwrap()
});

pub static LIBRA_STORAGE_LEDGER_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_storage_ledger_version",
        "Version in the latest saved ledger info."
    )
    .unwrap()
});

pub static LIBRA_STORAGE_NEXT_BLOCK_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_storage_next_block_epoch",
        "ledger_info.next_block_epoch() for the latest saved ledger info."
    )
    .unwrap()
});

pub static LIBRA_STORAGE_PRUNE_WINDOW: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("libra_storage_prune_window", "Libra storage prune window").unwrap()
});

pub static LIBRA_STORAGE_PRUNER_LEAST_READABLE_STATE_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_storage_pruner_least_readable_state_version",
        "Libra storage pruner least readable state version"
    )
    .unwrap()
});

pub static LIBRA_STORAGE_API_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "libra_storage_api_latency_seconds",
        // metric description
        "Libra storage api latency in seconds",
        // metric labels (dimensions)
        &["api_name"]
    )
    .unwrap()
});

pub static LIBRA_STORAGE_OTHER_TIMERS_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "libra_storage_other_timers_seconds",
        // metric description
        "Various timers below public API level.",
        // metric labels (dimensions)
        &["name"]
    )
    .unwrap()
});

// Backup progress gauges:

pub(crate) static BACKUP_EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_backup_handler_epoch_ending_epoch",
        "Current epoch returned in an epoch ending backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_backup_handler_transaction_version",
        "Current version returned in a transaction backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_backup_handler_state_snapshot_version",
        "Version of requested state snapshot backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_STATE_SNAPSHOT_LEAF_IDX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_backup_handler_state_snapshot_leaf_index",
        "Index of current leaf index returned in a state snapshot backup."
    )
    .unwrap()
});
