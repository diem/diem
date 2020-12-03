// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{register_int_counter, IntCounter};
use once_cell::sync::Lazy;

/// Count of the trace_event logged.
pub static TRACE_EVENT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("diem_trace_event_count", "Count of the trace_event logged.").unwrap()
});
