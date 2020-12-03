// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_secure_push_metrics::{register_int_gauge, IntGauge};
use once_cell::sync::Lazy;

pub static COORDINATOR_TARGET_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_coordinator_target_version",
        "The target version to restore to by the restore coordinator."
    )
    .unwrap()
});

pub static EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_epoch_ending_epoch",
        "Current epoch ending epoch being restored."
    )
    .unwrap()
});

pub static EPOCH_ENDING_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_epoch_ending_version",
        "Last version of the current epoch ending being restored."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_state_snapshot_version",
        "The version that a state snapshot restores to."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_TARGET_LEAF_INDEX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_state_snapshot_target_leaf_index",
        "The biggest leaf index in state snapshot being restored (# of accounts - 1)."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_LEAF_INDEX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_state_snapshot_leaf_index",
        "Current leaf index being restored."
    )
    .unwrap()
});

pub static TRANSACTION_SAVE_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_transaction_save_version",
        "Version of the transaction being restored without replaying."
    )
    .unwrap()
});

pub static TRANSACTION_REPLAY_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_transaction_replay_version",
        "Version of the transaction being replayed"
    )
    .unwrap()
});

pub static COORDINATOR_START_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_coordinator_start_timestamp_s",
        "Timestamp when the verify coordinator starts."
    )
    .unwrap()
});

pub static COORDINATOR_SUCC_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_coordinator_succeed_timestamp_s",
        "Timestamp when the verify coordinator fails."
    )
    .unwrap()
});

pub static COORDINATOR_FAIL_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_restore_coordinator_fail_timestamp_s",
        "Timestamp when the verify coordinator fails."
    )
    .unwrap()
});
