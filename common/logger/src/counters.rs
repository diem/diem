// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Logging metrics for determining quality of log submission
use once_cell::sync::Lazy;
use prometheus::{register_int_counter, IntCounter};

/// Count of the struct logs submitted by macro
pub static STRUCT_LOG_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("diem_struct_log_count", "Count of the struct logs.").unwrap()
});

/// Count of struct logs processed, but not necessarily sent
pub static PROCESSED_STRUCT_LOG_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_processed_count",
        "Count of the struct logs received by the sender."
    )
    .unwrap()
});

/// Count of struct logs submitted through TCP
pub static SENT_STRUCT_LOG_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_tcp_submit_count",
        "Count of the struct logs submitted by TCP."
    )
    .unwrap()
});

/// Number of bytes of struct logs submitted through TCP
pub static SENT_STRUCT_LOG_BYTES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_tcp_submit_bytes",
        "Number of bytes of the struct logs submitted by TCP."
    )
    .unwrap()
});

/// Metric for when we connect the outbound TCP
pub static STRUCT_LOG_TCP_CONNECT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_tcp_connect_count",
        "Count of the tcp connections made for struct logs."
    )
    .unwrap()
});

/// Metric for when we fail to log during sending to the queue
pub static STRUCT_LOG_QUEUE_ERROR_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_queue_error_count",
        "Count of all errors during queuing struct logs."
    )
    .unwrap()
});

/// Metric for when we fail to log during sending
pub static STRUCT_LOG_SEND_ERROR_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_send_error_count",
        "Count of all errors during sending struct logs."
    )
    .unwrap()
});

pub static STRUCT_LOG_CONNECT_ERROR_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_connect_error_count",
        "Count of all errors during connecting for struct logs."
    )
    .unwrap()
});

pub static STRUCT_LOG_PARSE_ERROR_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_struct_log_parse_error_count",
        "Count of all parse errors during struct logs."
    )
    .unwrap()
});
