// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_secure_push_metrics::{register_int_counter_vec, IntCounterVec};
use once_cell::sync::Lazy;

/// The metrics counter for the key manager.
static COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_key_manager_state",
        "Outcome for key operations",
        &["key", "state"]
    )
    .unwrap()
});

/// Metric counter keys.
const CHECK_KEYS: &str = "check_keys";
const CONSENSUS_KEY: &str = "consensus_key";

/// Metric counter states.
pub const KEYS_STILL_FRESH: &[&str] = &[CHECK_KEYS, "keys_still_fresh"];
pub const LIVENESS_ERROR_ENCOUNTERED: &[&str] = &[CHECK_KEYS, "liveness_error_encountered"];
pub const ROTATED_IN_STORAGE: &[&str] = &[CONSENSUS_KEY, "rotated_in_storage"];
pub const SUBMITTED_ROTATION_TRANSACTION: &[&str] =
    &[CONSENSUS_KEY, "submitted_rotation_transaction"];
pub const WAITING_ON_RECONFIGURATION: &[&str] = &[CHECK_KEYS, "waiting_on_reconfiguration"];
pub const WAITING_ON_TRANSACTION_EXECUTION: &[&str] =
    &[CHECK_KEYS, "waiting_on_transaction_execution"];
pub const UNEXPECTED_ERROR_ENCOUNTERED: &[&str] = &[CHECK_KEYS, "unexpected_error_encountered"];

/// Initializes all metric counter states.
pub fn initialize_all_metric_counters() {
    let metric_counter_states = &[
        KEYS_STILL_FRESH,
        LIVENESS_ERROR_ENCOUNTERED,
        ROTATED_IN_STORAGE,
        SUBMITTED_ROTATION_TRANSACTION,
        WAITING_ON_RECONFIGURATION,
        WAITING_ON_TRANSACTION_EXECUTION,
        UNEXPECTED_ERROR_ENCOUNTERED,
    ];
    let _ = metric_counter_states
        .iter()
        .for_each(|metric_counter_state| {
            COUNTER.with_label_values(metric_counter_state).reset();
        });
}

/// Increments a metric counter state.
pub fn increment_metric_counter(metric_counter_state: &[&str]) {
    COUNTER.with_label_values(metric_counter_state).inc();
}
