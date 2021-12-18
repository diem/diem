// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{
    register_histogram_vec, register_int_counter_vec, HistogramTimer, HistogramVec, IntCounterVec,
};
use network::ProtocolId;
use once_cell::sync::Lazy;

/// Counter for pending network events to the storage service (server-side).
pub static PENDING_STORAGE_SERVER_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_storage_service_server_pending_network_events",
        "Counters for pending network events for the storage server",
        &["state"]
    )
    .unwrap()
});

/// Counter for storage service errors encountered
pub static STORAGE_ERRORS_ENCOUNTERED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_storage_service_server_errors",
        "Counters related to the storage server errors encountered",
        &["protocol", "error_type"]
    )
    .unwrap()
});

/// Counter for received storage service requests
pub static STORAGE_REQUESTS_RECEIVED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_storage_service_server_requests_received",
        "Counters related to the storage server requests received",
        &["protocol", "request_type"]
    )
    .unwrap()
});

/// Counter for storage service responses sent
pub static STORAGE_RESPONSES_SENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_storage_service_server_responses_sent",
        "Counters related to the storage server responses sent",
        &["protocol", "response_type"]
    )
    .unwrap()
});

/// Time it takes to process a storage request
pub static STORAGE_REQUEST_PROCESSING_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_storage_service_server_request_latency",
        "Time it takes to process a storage service request",
        &["protocol", "request_type"]
    )
    .unwrap()
});

/// Increments the given counter with the provided label values.
pub fn increment_counter(counter: &Lazy<IntCounterVec>, protocol: ProtocolId, label: String) {
    counter
        .with_label_values(&[protocol.as_str(), &label])
        .inc();
}

/// Starts the timer for the provided histogram and label values.
pub fn start_timer(
    histogram: &Lazy<HistogramVec>,
    protocol: ProtocolId,
    label: String,
) -> HistogramTimer {
    histogram
        .with_label_values(&[protocol.as_str(), &label])
        .start_timer()
}
