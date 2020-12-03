// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_secure_push_metrics::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, HistogramTimer,
    HistogramVec, IntCounterVec, IntGaugeVec,
};
use once_cell::sync::Lazy;

pub static LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_safety_rules_latency",
        "Time to perform an operation",
        &["source", "field"]
    )
    .unwrap()
});

static QUERY_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_safety_rules_queries",
        "Outcome of calling into LSR",
        &["method", "result"]
    )
    .unwrap()
});

static STATE_GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_safety_rules_state",
        "Current internal state of LSR",
        &["field"]
    )
    .unwrap()
});

pub fn increment_query(method: &str, result: &str) {
    QUERY_COUNTER.with_label_values(&[method, result]).inc();
}

pub fn start_timer(source: &str, field: &str) -> HistogramTimer {
    LATENCY.with_label_values(&[source, field]).start_timer()
}

pub fn set_state(field: &str, value: i64) {
    STATE_GAUGE.with_label_values(&[field]).set(value);
}
