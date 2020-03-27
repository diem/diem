// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;
use libra_secure_push_metrics::{define_counters, Counter, Gauge};
use std::sync::Arc;

// @TODO This is still Work in Progress and not ready for production
// Use the libra_safety_rules prefix for all counters
define_counters![
    "libra_safety_rules",
    (
        sign_proposal: Counter,
        "sign_proposal counter counts sign_proposals"
    ),
    (
        sign_timeout: Counter,
        "sign_timeout counter counts sign_timeouts"
    ),
    (some_gauge_counter: Gauge, "example help for a gauge metric"),
];

lazy_static! {
    pub static ref COUNTERS: Arc<Counters> = Arc::new(Counters::new());
}
