// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{
    register_histogram_vec, register_int_counter, register_int_gauge, register_int_gauge_vec,
    HistogramVec, IntCounter, IntGauge, IntGaugeVec,
};
use once_cell::sync::Lazy;

pub static DIEM_STORAGE_LEDGER: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "diem_storage_ledger",
        // metric description
        "Diem storage ledger counters",
        // metric labels (dimensions)
        &["type"]
    )
    .unwrap()
});

pub static DIEM_STORAGE_CF_SIZE_BYTES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "diem_storage_cf_size_bytes",
        // metric description
        "Diem storage Column Family size in bytes",
        // metric labels (dimensions)
        &["cf_name"]
    )
    .unwrap()
});

pub static DIEM_STORAGE_COMMITTED_TXNS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_storage_committed_txns",
        "Diem storage committed transactions"
    )
    .unwrap()
});

pub static DIEM_STORAGE_LATEST_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_storage_latest_transaction_version",
        "Diem storage latest transaction version"
    )
    .unwrap()
});

pub static DIEM_STORAGE_LEDGER_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_storage_ledger_version",
        "Version in the latest saved ledger info."
    )
    .unwrap()
});

pub static DIEM_STORAGE_NEXT_BLOCK_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_storage_next_block_epoch",
        "ledger_info.next_block_epoch() for the latest saved ledger info."
    )
    .unwrap()
});

pub static DIEM_STORAGE_PRUNE_WINDOW: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("diem_storage_prune_window", "Diem storage prune window").unwrap()
});

pub static DIEM_STORAGE_PRUNER_LEAST_READABLE_STATE_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_storage_pruner_least_readable_state_version",
        "Diem storage pruner least readable state version"
    )
    .unwrap()
});

pub static DIEM_STORAGE_API_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "diem_storage_api_latency_seconds",
        // metric description
        "Diem storage api latency in seconds",
        // metric labels (dimensions)
        &["api_name", "result"]
    )
    .unwrap()
});

pub static DIEM_STORAGE_OTHER_TIMERS_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "diem_storage_other_timers_seconds",
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
        "diem_backup_handler_epoch_ending_epoch",
        "Current epoch returned in an epoch ending backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_backup_handler_transaction_version",
        "Current version returned in a transaction backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_backup_handler_state_snapshot_version",
        "Version of requested state snapshot backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_STATE_SNAPSHOT_LEAF_IDX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_backup_handler_state_snapshot_leaf_index",
        "Index of current leaf index returned in a state snapshot backup."
    )
    .unwrap()
});
