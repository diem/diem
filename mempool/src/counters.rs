// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, HistogramVec,
    IntCounterVec, IntGaugeVec,
};
use once_cell::sync::Lazy;

// Core mempool index labels
pub const PRIORITY_INDEX_LABEL: &str = "priority";
pub const EXPIRATION_TIME_INDEX_LABEL: &str = "expiration";
pub const SYSTEM_TTL_INDEX_LABEL: &str = "system_ttl";
pub const TIMELINE_INDEX_LABEL: &str = "timeline";
pub const PARKING_LOT_INDEX_LABEL: &str = "parking_lot";

// Mempool service request type labels
pub const GET_BLOCK_LABEL: &str = "get_block";
pub const COMMIT_STATE_SYNC_LABEL: &str = "commit_accepted";
pub const COMMIT_CONSENSUS_LABEL: &str = "commit_rejected";

// Mempool service request result labels
pub const REQUEST_FAIL_LABEL: &str = "fail";
pub const REQUEST_SUCCESS_LABEL: &str = "success";

// Process txn breakdown type labels
pub const FETCH_SEQ_NUM_LABEL: &str = "storage_fetch";
pub const VM_VALIDATION_LABEL: &str = "vm_validation";

// Bounded executor task labels
pub const CLIENT_EVENT_LABEL: &str = "client_event";
pub const STATE_SYNC_EVENT_LABEL: &str = "state_sync";
pub const RECONFIG_EVENT_LABEL: &str = "reconfig";
pub const PEER_BROADCAST_EVENT_LABEL: &str = "peer_broadcast";

/// Counter tracking size of various indices in core mempool
pub static CORE_MEMPOOL_INDEX_SIZE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_core_mempool_index_size",
        "Size of a core mempool index",
        &["index"]
    )
    .unwrap()
});

/// Counter of pending network events to Mempool
pub static PENDING_MEMPOOL_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "mempool_pending_network_events",
        "Counters(queued,dequeued,dropped) related to pending network notifications to Mempool",
        &["state"]
    )
    .unwrap()
});

/// Counter of number of txns processed in each consensus/state sync message
/// (e.g. # txns in block pulled by consensus, # txns committed from state sync)
pub static MEMPOOL_SERVICE_TXNS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_mempool_service_transactions",
        "Number of transactions handled in one request/response between mempool and consensus/state sync",
        &["type"]
    )
        .unwrap()
});

/// Counter for tracking latency of mempool processing requests from consensus/state sync
pub static MEMPOOL_SERVICE_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_mempool_service_latency_ms",
        "Latency of mempool processing request from consensus/state sync",
        &["type", "result"]
    )
    .unwrap()
});

pub static SHARED_MEMPOOL_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_shared_mempool_events",
        "Number of network events received by shared mempool",
        &["event"] // type of event: "new_peer", "lost_peer", "message"
    )
    .unwrap()
});

/// Counter for tracking e2e latency for mempool to process txn submission requests from clients and peers
pub static PROCESS_TXN_SUBMISSION_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_shared_mempool_request_latency",
        "Latency of mempool processing txn submission requests",
        &["sender"] // sender of txn(s)
    )
    .unwrap()
});

/// Tracks latency of different stages of txn processing (e.g. vm validation, storage read)
pub static PROCESS_TXN_BREAKDOWN_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_mempool_process_txn_breakdown_latency",
        "Latency of different stages of processing txns in mempool",
        &["portion"]
    )
    .unwrap()
});

/// Counter for tracking latency for mempool to broadcast to a peer
pub static SHARED_MEMPOOL_BROADCAST_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_broadcast_latency",
        "Latency of mempool executing broadcast to another peer",
        &["sender"]
    )
    .unwrap()
});

/// Counter for tracking roundtrip-time from sending a broadcast to receiving ACK for that broadcast
pub static SHARED_MEMPOOL_BROADCAST_RTT: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_shared_mempool_broadcast_roundtrip_latency",
        "Time elapsed between sending a broadcast and receiving an ACK for that broadcast",
        &["recipient", "broadcast_id"]
    )
    .unwrap()
});

/// Counter tracking number of mempool broadcasts that have not been ACK'ed for
pub static SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_shared_mempool_pending_broadcasts_count",
        "Number of mempool broadcasts not ACK'ed for yet",
        &["recipient", "broadcast_id"]
    )
    .unwrap()
});

pub static SHARED_MEMPOOL_TRANSACTIONS_PROCESSED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_shared_mempool_transactions_processed",
        "Number of transactions received and handled by shared mempool",
        &[
            // state of transaction processing: "received", "success", status code from failed txn processing
            "status", // sender of the txns
            "sender"
        ]
    )
    .unwrap()
});

// Counter for broadcast size to peers
pub static SHARED_MEMPOOL_TRANSACTION_BROADCAST: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_shared_mempool_transaction_broadcast",
        "Number of transactions in each mempool broadcast sent",
        &["recipient"]
    )
    .unwrap()
});

pub static TASK_SPAWN_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_bounded_executor_spawn_latency",
        "Time it takes for mempool's coordinator to spawn async tasks",
        &["task"]
    )
    .unwrap()
});
