// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_secure_push_metrics::{register_int_gauge, IntGauge};
use once_cell::sync::Lazy;

pub static COORDINATOR_TARGET_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_coordinator_target_version",
        "The target version to restore to by the restore coordinator."
    )
    .unwrap()
});

pub static EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_epoch_ending_epoch",
        "Current epoch ending epoch being restored."
    )
    .unwrap()
});

pub static EPOCH_ENDING_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_epoch_ending_version",
        "Last version of the current epoch ending being restored."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_state_snapshot_version",
        "The version that a state snapshot restores to."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_TOTAL_LEAVES: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_state_snapshot_total_leaves",
        "Number of account in the state snapshot being restored."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_LEAF_INDEX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_state_snapshot_leaf_index",
        "Current leaf index being restored."
    )
    .unwrap()
});

pub static TRANSACTION_SAVE_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_transaction_save_version",
        "Version of the transaction being restored without replaying."
    )
    .unwrap()
});

pub static TRANSACTION_REPLAY_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_db_restore_transaction_replay_version",
        "Version of the transaction being replayed"
    )
    .unwrap()
});
