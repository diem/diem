// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_secure_push_metrics::{register_int_gauge, IntGauge};
use once_cell::sync::Lazy;

pub static HEARTBEAT_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_coordinator_heartbeat_timestamp_s",
        "Timestamp when the backup coordinator successfully updates state from the backup service."
    )
    .unwrap()
});

pub static EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_coordinator_epoch_ending_epoch",
        "Epoch of the latest epoch ending backed up."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_coordinator_state_snapshot_version",
        "The version of the latest state snapshot taken."
    )
    .unwrap()
});

pub static TRANSACTION_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_db_backup_coordinator_transaction_version",
        "Version of the latest transaction backed up."
    )
    .unwrap()
});
