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
        last_voted_round: Gauge,
        "The round of the highest voted block"
    ),
    (
        preferred_round: Gauge,
        "The round of the highest 2-chain head"
    ),
    (
        sign_proposal_request: Counter,
        "The number of requests to sign_proposal"
    ),
    (
        sign_proposal_success: Counter,
        "The number of successful requests to sign_proposal"
    ),
    (
        sign_timeout_request: Counter,
        "The number of requests to sign_timeout"
    ),
    (
        sign_timeout_success: Counter,
        "The number of successful requests to sign_timeout"
    ),
];

pub static COUNTERS: Lazy<Arc<Counters>> = Lazy::new(|| Arc::new(Counters::new()));
