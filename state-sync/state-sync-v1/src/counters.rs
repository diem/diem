// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, DurationHistogram, Histogram, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
};
use once_cell::sync::Lazy;

// network send result labels
pub const SEND_SUCCESS_LABEL: &str = "success";
pub const SEND_FAIL_LABEL: &str = "fail";

// msg type labels
pub const SYNC_MSG_LABEL: &str = "sync";
pub const COMMIT_MSG_LABEL: &str = "commit";
pub const CHUNK_REQUEST_MSG_LABEL: &str = "chunk_request";
pub const CHUNK_RESPONSE_MSG_LABEL: &str = "chunk_response";

pub fn set_timestamp(timestamp_type: TimestampType, time_as_usecs: u64) {
    TIMESTAMP
        .with_label_values(&[timestamp_type.as_str()])
        .set((time_as_usecs / 1000) as i64)
}

pub enum TimestampType {
    /// Current ledger committed timestamp
    Committed,
    /// Current computers clock
    Real,
    /// Current ledger synced timestamp
    Synced,
}

impl TimestampType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimestampType::Committed => "committed",
            TimestampType::Real => "real",
            TimestampType::Synced => "synced",
        }
    }
}

pub fn set_version(version_type: VersionType, version: u64) {
    VERSION
        .with_label_values(&[version_type.as_str()])
        .set(version as i64)
}

pub fn get_version(version_type: VersionType) -> u64 {
    VERSION.with_label_values(&[version_type.as_str()]).get() as u64
}

pub enum VersionType {
    /// Version of latest ledger info committed.
    Committed,
    /// Highest known version or version proceeding it
    Highest,
    /// Version of most recent txn that was synced (even if it is not backed by an LI)
    Synced,
    /// Current version a node is trying to catch up to usually within the current epoch
    Target,
}

impl VersionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VersionType::Committed => "committed",
            VersionType::Highest => "highest",
            VersionType::Synced => "synced",
            VersionType::Target => "target",
        }
    }
}

// failed channel send type labels
pub const CONSENSUS_SYNC_REQ_CALLBACK: &str = "consensus_sync_req_callback";
pub const WAYPOINT_INIT_CALLBACK: &str = "waypoint_init_callback";

// result labels
pub const SUCCESS_LABEL: &str = "success";
pub const FAIL_LABEL: &str = "fail";

// commit flow fail component label
pub const MEMPOOL_LABEL: &str = "mempool";
pub const CONSENSUS_LABEL: &str = "consensus";

// sync request result labels
pub const COMPLETE_LABEL: &str = "complete";
pub const TIMEOUT_LABEL: &str = "timeout";

/// Counter of pending network events to State Sync
pub static PENDING_STATE_SYNC_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_pending_network_events",
        "Counters(queued,dequeued,dropped) related to pending network notifications for State Sync",
        &["state"]
    )
    .unwrap()
});

/// Number of chunk requests sent from a node
pub static REQUESTS_SENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        // metric name
        "diem_state_sync_requests_sent_total",
        // metric description
        "Number of chunk requests sent",
        // metric labels
        &["network", "peer", "result"]
    )
    .unwrap()
});

/// Number of chunk responses sent from a node (including FN subscriptions)
pub static RESPONSES_SENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_responses_sent_total",
        "Number of chunk responses sent (including FN subscriptions)",
        &["network", "peer", "result"]
    )
    .unwrap()
});

pub static RESPONSE_FROM_DOWNSTREAM_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_responses_from_downstream_total",
        "Number of chunk responses received from a downstream peer",
        &["network", "peer"]
    )
    .unwrap()
});

/// Number of attempts to apply a chunk
pub static APPLY_CHUNK_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_apply_chunk_total",
        "Number of Success results of applying a chunk",
        &["network", "sender", "result"]
    )
    .unwrap()
});

pub static PROCESS_CHUNK_REQUEST_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_process_chunk_request_total",
        "Number of times chunk request was processed",
        &["network", "sender", "result"]
    )
    .unwrap()
});

/// Number of transactions in a received chunk response
pub static STATE_SYNC_CHUNK_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_state_sync_chunk_size",
        "Number of transactions in a state sync chunk response",
        &["network", "sender"]
    )
    .unwrap()
});

/// Number of peers that are currently active and upstream.
/// They are the set of nodes a node can make sync requests to
pub static ACTIVE_UPSTREAM_PEERS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_state_sync_active_upstream_peers",
        "Number of upstream peers that are currently active",
        &["network"]
    )
    .unwrap()
});

/// Highest preference of the networks this node is sending chunk requests to.
/// It is usually 0 if the node's primary network is healthy, but can be >0 if the node's primary
/// network is unhealthy/all peers in that network are dead
/// and the node fails over to other networks
pub static MULTICAST_LEVEL: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_state_sync_multicast_level",
        "Max network preference of the networks state sync is sending chunk requests to"
    )
    .unwrap()
});

pub static TIMESTAMP: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_state_sync_timestamp",
        "Timestamp involved in state sync progress",
        &["type"] // see TimestampType above
    )
    .unwrap()
});

/// Notice: this metric is used in CT full node health check
/// ~/diem/testsuite/cluster-test/health/fullnode_check.rs
/// please make corresponding changes if this field is updated
pub static VERSION: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_state_sync_version",
        "Version involved in state sync progress",
        &["type"] // see version type labels above
    )
    .unwrap()
});

pub static EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("diem_state_sync_epoch", "Current epoch in local state").unwrap()
});

/// How long it takes to make progress, from requesting a chunk to processing the response and
/// committing the block
pub static SYNC_PROGRESS_DURATION: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "diem_state_sync_sync_progress_duration_s",
            "Histogram of time it takes to sync a chunk, from requesting a chunk to processing the response and committing the chunk"
        )
        .unwrap()
    )
});

/// Number of timeouts that occur during sync
pub static TIMEOUT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_state_sync_timeout_total",
        "Number of timeouts that occur during sync"
    )
    .unwrap()
});

/// Number of times sync request (from consensus) processed
pub static SYNC_REQUEST_RESULT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_sync_request_total",
        "Number of sync requests (from consensus) processed",
        &["result"]
    )
    .unwrap()
});

/// Number of failures that occur during the commit flow across consensus, state sync, and mempool
pub static COMMIT_FLOW_FAIL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_commit_flow_fail_total",
        "Number of timeouts that occur during the commit flow across consensus, state sync, and mempool",
        &["component"] // component with which state sync timed out with: consensus, to_mempool, from_mempool
    )
        .unwrap()
});

pub static FAILED_CHANNEL_SEND: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_failed_channel_sends_total",
        "Number of times a channel send failed in state sync",
        &["type"]
    )
    .unwrap()
});

/// Time it takes for state sync to fully execute a chunk (via executor proxy)
pub static EXECUTE_CHUNK_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "diem_state_sync_execute_chunk_duration_s",
        "Histogram of time it takes for state sync's executor proxy to fully execute a chunk"
    )
    .unwrap()
});

/// Number of times a long-poll subscription is successfully delivered
pub static SUBSCRIPTION_DELIVERY_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_subscription_delivery_count",
        "Number of times a node delivers a subscription for FN long-poll",
        &["network", "recipient", "result"]
    )
    .unwrap()
});

/// Time it takes to process a coordinator msg from consensus
pub static PROCESS_COORDINATOR_MSG_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_state_sync_coordinator_msg_latency",
        "Time it takes to process a message from consensus",
        &["type"]
    )
    .unwrap()
});

/// Time it takes to process a state sync message from DiemNet
pub static PROCESS_MSG_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_state_sync_process_msg_latency",
        "Time it takes to process a message in state sync",
        &["network", "sender", "type"]
    )
    .unwrap()
});

pub static CONSENSUS_COMMIT_FAIL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_state_sync_consensus_commit_fail",
        "Number of times a commit msg from consensus failed to be processed"
    )
    .unwrap()
});

pub static RECONFIG_PUBLISH_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_state_sync_reconfig_count",
        "Number of times on-chain reconfig notification is published in state sync",
        &["result"]
    )
    .unwrap()
});

pub static STORAGE_READ_FAIL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_state_sync_storage_read_fail_count",
        "Number of times storage read failed in state sync"
    )
    .unwrap()
});

pub static NETWORK_ERROR_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_state_sync_network_error_count",
        "Number of network errors encountered in state sync"
    )
    .unwrap()
});

/// Duration of each run of the event loop.
pub static MAIN_LOOP: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "diem_state_sync_main_loop",
            "Duration of the each run of the event loop"
        )
        .unwrap(),
    )
});
