// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_secure_push_metrics::{define_counters, Counter, Gauge};
use once_cell::sync::Lazy;
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

pub static COUNTERS: Lazy<Arc<Counters>> = Lazy::new(|| Arc::new(Counters::new()));
