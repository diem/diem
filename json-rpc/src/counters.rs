// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use prometheus::IntGaugeVec;

/// Cumulative number of valid requests that the JSON RPC client service receives
pub static REQUESTS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_client_service_requests_count",
        "Cumulative number of requests that JSON RPC client service receives",
        &[
            "type",   // type of request, matches JSON RPC method name (e.g. "submit", "get_account_state")
            "result", // result of request: "success", "fail"
        ]
    )
    .unwrap()
});

/// Cumulative number of invalid requests that the JSON RPC client service receives
pub static INVALID_REQUESTS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_client_service_invalid_requests_count",
        "Cumulative number of invalid requests that JSON RPC client service receives",
        &[
            "type", // categories of invalid requests: "invalid_format", "invalid_params", "invalid_method", "method_not_found"
        ]
    )
    .unwrap()
});
