// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{register_int_counter, register_int_counter_vec, IntCounter, IntCounterVec};
use once_cell::sync::Lazy;

/// Cumulative number of valid requests that the JSON RPC client service receives
pub static REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_client_service_requests_count",
        "Cumulative number of requests that JSON RPC client service receives",
        &[
            "type",   // type of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "result", // result of request: "success", "fail"
        ]
    )
    .unwrap()
});

/// Cumulative number of invalid requests that the JSON RPC client service receives
pub static INVALID_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_client_service_invalid_requests_count",
        "Cumulative number of invalid requests that JSON RPC client service receives",
        &[
            "type", // categories of invalid requests: "invalid_format", "invalid_params", "invalid_method", "method_not_found"
        ]
    )
    .unwrap()
});

/// Cumulative number of server internal errors.
pub static INTERNAL_ERRORS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_client_service_internal_error_count",
        "Cumulative number of internal error"
    )
    .unwrap()
});
