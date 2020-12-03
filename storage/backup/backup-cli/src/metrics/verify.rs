// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_secure_push_metrics::{register_int_gauge, IntGauge};
use once_cell::sync::Lazy;

pub static VERIFY_EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_epoch_ending_epoch",
        "Current epoch ending epoch being verified."
    )
    .unwrap()
});

pub static VERIFY_EPOCH_ENDING_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_epoch_ending_version",
        "Last version of the current epoch ending being verified."
    )
    .unwrap()
});

pub static VERIFY_STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_state_snapshot_version",
        "The version of the verified state snapshot."
    )
    .unwrap()
});

pub static VERIFY_STATE_SNAPSHOT_TARGET_LEAF_INDEX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_state_snapshot_target_leaf_index",
        "The biggest leaf index in state snapshot being verified (# of accounts - 1)."
    )
    .unwrap()
});

pub static VERIFY_STATE_SNAPSHOT_LEAF_INDEX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_state_snapshot_leaf_index",
        "Current leaf index being verified."
    )
    .unwrap()
});

pub static VERIFY_TRANSACTION_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_transaction_version",
        "Version of the transaction being verified."
    )
    .unwrap()
});

pub static VERIFY_COORDINATOR_START_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_coordinator_start_timestamp_s",
        "Timestamp when the verify coordinator starts."
    )
    .unwrap()
});

pub static VERIFY_COORDINATOR_SUCC_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_coordinator_succeed_timestamp_s",
        "Timestamp when the verify coordinator fails."
    )
    .unwrap()
});

pub static VERIFY_COORDINATOR_FAIL_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_verify_coordinator_fail_timestamp_s",
        "Timestamp when the verify coordinator fails."
    )
    .unwrap()
});
