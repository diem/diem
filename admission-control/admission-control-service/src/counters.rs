// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{register_int_gauge_vec, IntGaugeVec};
use once_cell::sync::Lazy;

/// Cumulative number of requests that admission control service receives
pub static REQUESTS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_admission_control_service_requests_count",
        "Cumulative number of requests that admission control receives",
        &["type"] // type of request: "submit_transaction", "update_to_latest_ledger"
    )
    .unwrap()
});
