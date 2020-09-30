// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_secure_push_metrics::{register_int_counter_vec, IntCounterVec};
use once_cell::sync::Lazy;

static STATE_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_key_manager_state",
        "Outcome for key operations",
        &["key", "state"]
    )
    .unwrap()
});

pub fn increment_state(key: &str, state: &str) {
    STATE_COUNTER.with_label_values(&[key, state]).inc();
}
