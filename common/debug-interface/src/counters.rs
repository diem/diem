// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{register_int_counter, IntCounter};
use once_cell::sync::Lazy;

/// Count of the trace_event logged.
pub static TRACE_EVENT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_trace_event_count",
        "Count of the trace_event logged."
    )
    .unwrap()
});

/// Count of the trace_edge logged.
pub static TRACE_EDGE_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_trace_edge_count", "Count of the trace_edge logged.").unwrap()
});

/// Count of the end_trace logged.
pub static END_TRACE_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_end_trace_count", "Count of the end_trace logged.").unwrap()
});
