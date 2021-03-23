// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};
use once_cell::sync::Lazy;

/// Cumulative number of rpc requests that the JSON RPC service receives
pub static RPC_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_client_service_rpc_requests_count",
        "Cumulative number of rpc requests that JSON RPC client service receives",
        &["type"] // batch / single
    )
    .unwrap()
});

pub static RPC_REQUEST_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_client_service_rpc_request_latency_seconds",
        "Diem client service rpc request latency histogram",
        &["type"] // batch / single
    )
    .unwrap()
});

/// Cumulative number of valid requests that the JSON RPC client service receives
pub static REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_client_service_requests_count",
        "Cumulative number of requests that JSON RPC client service receives",
        &[
            "type",     // batch / single
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "result", // result of request: "success", "fail"
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver", // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

/// Cumulative number of invalid requests that the JSON RPC client service receives
pub static INVALID_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_client_service_invalid_requests_count",
        "Cumulative number of invalid requests that JSON RPC client service receives",
        &[
            "type",      // batch / single
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "errortype", // categories of invalid requests: "invalid_format", "invalid_params", "invalid_method", "method_not_found"
            "sdk_lang",  // the language of the SDK: "java", "python", etc
            "sdk_ver",   // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

/// Cumulative number of server internal errors.
pub static INTERNAL_ERRORS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_client_service_internal_error_count",
        "Cumulative number of internal error",
        &[
            "type",      // batch / single
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "errorcode", // error code
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver", // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

pub static METHOD_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_client_service_method_latency_seconds",
        "Diem client service method latency histogram",
        &[
            "type",   // batch / single
            "method"  // JSON-RPC methods: submit, get_account ...
        ]
    )
    .unwrap()
});
