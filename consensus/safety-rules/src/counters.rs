// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_secure_push_metrics::{
    register_int_counter_vec, register_int_gauge_vec, IntCounterVec, IntGaugeVec,
};
use once_cell::sync::Lazy;

static QUERY_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_safety_rules_queries",
        "Outcome of calling into LSR",
        &["method", "result"]
    )
    .unwrap()
});

static STATE_GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_safety_rules_state",
        "Current internal state of LSR",
        &["field"]
    )
    .unwrap()
});

pub fn increment_query(method: &str, result: &str) {
    QUERY_COUNTER.with_label_values(&[method, result]).inc();
}

pub fn set_state(field: &str, value: i64) {
    STATE_GAUGE.with_label_values(&[field]).set(value);
}
