// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::DurationHistogram;
use once_cell::sync::Lazy;
use prometheus::{IntCounter, IntCounterVec, IntGauge};

/// Counter of pending network events to State Synchronizer
pub static PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_state_sync_pending_network_events",
        "Counters(queued,dequeued,dropped) related to pending network notifications for State Synchronizer",
        &["state"]
    ).unwrap()
});

/// Number of sync requests sent from a node
pub static REQUESTS_SENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        // metric name
        "libra_state_sync_requests_sent_total",
        // metric description
        "Number of sync requests sent from a node",
        // metric labels
        &["requested_peer_id"]
    )
    .unwrap()
});

/// Number of sync responses a node received
pub static RESPONSES_RECEIVED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_state_sync_responses_received_total",
        "Number of sync responses a node received",
        &["response_sender_id"]
    )
    .unwrap()
});

/// Number of Success results of applying a chunk
pub static APPLY_CHUNK_SUCCESS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_state_sync_apply_chunk_success_total",
        "Number of Success results of applying a chunk",
        &["chunk_sender_id"]
    )
    .unwrap()
});

/// Number of failed attempts to apply a chunk
pub static APPLY_CHUNK_FAILURE: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_state_sync_apply_chunk_failure_total",
        "Number of failed attempts to apply a chunk",
        &["chunk_sender_id"]
    )
    .unwrap()
});

/// Count the overall number of transactions state synchronizer has retrieved since last restart.
/// Large values mean that a node has been significantly behind and had to replay a lot of txns.
pub static STATE_SYNC_TXN_REPLAYED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_state_sync_txns_replayed_total",
        "Number of transactions the state synchronizer has retrieved since last restart"
    )
    .unwrap()
});

/// Number of peers that are currently active and upstream.
/// They are the set of nodes a node can make sync requests to
pub static ACTIVE_UPSTREAM_PEERS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_state_sync_active_upstream_peers",
        "Number of upstream peers that are currently active"
    )
    .unwrap()
});

/// Most recent version that has been committed
pub static COMMITTED_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_state_sync_committed_version",
        "Most recent version that has been committed"
    )
    .unwrap()
});

/// Most recent version that has been committed
pub static COMMITTED_TIMESTAMP: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_state_sync_committed_timestamp",
        "Most recent timestamp that has been committed"
    )
    .unwrap()
});

/// How long it takes to make progress, from requesting a chunk to processing the response and
/// committing the block
pub static SYNC_PROGRESS_DURATION: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_state_sync_sync_progress_duration_s",
            "Histogram of time it takes to sync a chunk, from requesting a chunk to processing the response and committing the block"
        )
        .unwrap()
    )
});

/// Version a node is trying to catch up to
pub static TARGET_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_state_sync_target_version",
        "Version a node is trying to catch up to"
    )
    .unwrap()
});

/// Number of timeouts that occur during sync
pub static TIMEOUT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_state_sync_timeout_total",
        "Number of timeouts that occur during sync"
    )
    .unwrap()
});
