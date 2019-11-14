// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use libra_metrics::DurationHistogram;
use prometheus::{IntCounter, IntCounterVec, IntGauge};

lazy_static::lazy_static! {
    /// Number of sync requests sent from a node
    pub static ref REQUESTS_SENT: IntCounterVec = register_int_counter_vec!(
        // metric name
        "libra_state_sync_requests_sent_total",
        // metric description
        "Number of sync requests sent from a node",
        // metric labels
        &["requested_peer_id"]
    ).unwrap();

    /// Number of sync responses a node received
    pub static ref RESPONSES_RECEIVED: IntCounterVec = register_int_counter_vec!(
        "libra_state_sync_responses_received_total",
        "Number of sync responses a node received",
        &["response_sender_id"]
    ).unwrap();

    /// Number of Success results of applying a chunk
    pub static ref APPLY_CHUNK_SUCCESS: IntCounterVec = register_int_counter_vec!(
        "libra_state_sync_apply_chunk_success_total",
        "Number of Success results of applying a chunk",
        &["chunk_sender_id"]
    ).unwrap();

    /// Number of failed attempts to apply a chunk
    pub static ref APPLY_CHUNK_FAILURE: IntCounterVec = register_int_counter_vec!(
        "libra_state_sync_apply_chunk_failure_total",
        "Number of failed attempts to apply a chunk",
        &["chunk_sender_id"]
    ).unwrap();

    /// Count the overall number of transactions state synchronizer has retrieved since last restart.
    /// Large values mean that a node has been significantly behind and had to replay a lot of txns.
    pub static ref STATE_SYNC_TXN_REPLAYED: IntCounter = register_int_counter!(
        "libra_state_sync_txns_replayed_total",
        "Number of transactions the state synchronizer has retrieved since last restart"
    ).unwrap();

    /// Number of peers that are currently active and upstream.
    /// They are the set of nodes a node can make sync requests to
    pub static ref ACTIVE_UPSTREAM_PEERS: IntGauge = register_int_gauge!(
        "libra_state_sync_active_upstream_peers",
        "Number of upstream peers that are currently active"
    ).unwrap();

    /// Most recent version that has been committed
    pub static ref COMMITTED_VERSION: IntGauge = register_int_gauge!(
        "libra_state_sync_committed_version",
        "Most recent version that has been committed"
    ).unwrap();

    /// How long it takes to make progress, from requesting a chunk to processing the response and
    /// committing the block
    pub static ref SYNC_PROGRESS_DURATION: DurationHistogram = DurationHistogram::new(
        register_histogram!(
            "libra_state_sync_sync_progress_duration_s",
            "Histogram of time it takes to sync a chunk, from requesting a chunk to processing the response and committing the block"
        )
        .unwrap()
    );

    /// Version a node is trying to catch up to
    pub static ref TARGET_VERSION: IntGauge = register_int_gauge!(
        "libra_state_sync_target_version",
        "Version a node is trying to catch up to"
    ).unwrap();

    /// Number of timeouts that occur during sync
    pub static ref TIMEOUT: IntCounter = register_int_counter!(
        "libra_state_sync_timeout_total",
        "Number of timeouts that occur during sync"
    ).unwrap();
}
