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

/// Count of the call to trace_event! macro.
pub static TRACE_EVENT_CALL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_trace_event_call_count",
        "Count of the call to trace_event! macro."
    )
    .unwrap()
});

/// Count of trace_event selected.
pub static TRACE_EVENT_SELECT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_trace_event_select_count",
        "Count of trace_event selected."
    )
    .unwrap()
});

/// Count of the call to drop.
pub static TRACE_EVENT_DROP_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_trace_event_drop_count", "Count of the call to drop.").unwrap()
});
