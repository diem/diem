// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_config::{config::PeerNetworkId, network_id::NetworkId};
use diem_metrics::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, DurationHistogram, HistogramTimer, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
};
use diem_types::PeerId;
use once_cell::sync::Lazy;
use short_hex_str::AsShortHexStr;
use std::time::Duration;

// Core mempool index labels
pub const PRIORITY_INDEX_LABEL: &str = "priority";
pub const EXPIRATION_TIME_INDEX_LABEL: &str = "expiration";
pub const SYSTEM_TTL_INDEX_LABEL: &str = "system_ttl";
pub const TIMELINE_INDEX_LABEL: &str = "timeline";
pub const PARKING_LOT_INDEX_LABEL: &str = "parking_lot";

// Core mempool commit stages labels
pub const GET_BLOCK_STAGE_LABEL: &str = "get_block";
pub const COMMIT_ACCEPTED_LABEL: &str = "commit_accepted";
pub const COMMIT_REJECTED_LABEL: &str = "commit_rejected";

// Core mempool GC type labels
pub const GC_SYSTEM_TTL_LABEL: &str = "system_ttl";
pub const GC_CLIENT_EXP_LABEL: &str = "client_expiration";

// Core mempool GC txn status label
pub const GC_ACTIVE_TXN_LABEL: &str = "active";
pub const GC_PARKED_TXN_LABEL: &str = "parked";

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

// Txn process result labels
pub const CLIENT_LABEL: &str = "client";
pub const SUCCESS_LABEL: &str = "success";

// Bounded executor task labels
pub const CLIENT_EVENT_LABEL: &str = "client_event";
pub const RECONFIG_EVENT_LABEL: &str = "reconfig";
pub const PEER_BROADCAST_EVENT_LABEL: &str = "peer_broadcast";

// task spawn stage labels
pub const SPAWN_LABEL: &str = "spawn";
pub const START_LABEL: &str = "start";

// Mempool network msg failure type labels:
pub const BROADCAST_TXNS: &str = "broadcast_txns";
pub const ACK_TXNS: &str = "ack_txns";

// Broadcast/ACK type labels
pub const EXPIRED_BROADCAST_LABEL: &str = "expired";
pub const RETRY_BROADCAST_LABEL: &str = "retry";
pub const BACKPRESSURE_BROADCAST_LABEL: &str = "backpressure";

// ACK direction labels
pub const RECEIVED_LABEL: &str = "received";
pub const SENT_LABEL: &str = "sent";

// invalid ACK type labels
pub const INVALID_REQUEST_ID: &str = "invalid_req_id";
pub const UNKNOWN_PEER: &str = "unknown_peer";

/// Counter tracking size of various indices in core mempool
static CORE_MEMPOOL_INDEX_SIZE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_core_mempool_index_size",
        "Size of a core mempool index",
        &["index"]
    )
    .unwrap()
});

pub fn core_mempool_index_size(label: &'static str, size: usize) {
    CORE_MEMPOOL_INDEX_SIZE
        .with_label_values(&[label])
        .set(size as i64)
}

/// Counter tracking number of txns removed from core mempool
pub static CORE_MEMPOOL_REMOVED_TXNS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_core_mempool_removed_txns_count",
        "Number of txns removed from core mempool"
    )
    .unwrap()
});

/// Counter tracking latency of txns reaching various stages in committing
/// (e.g. time from txn entering core mempool to being pulled in consensus block)
pub static CORE_MEMPOOL_TXN_COMMIT_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_core_mempool_txn_commit_latency",
        "Latency of txn reaching various stages in core mempool after insertion",
        &["stage"]
    )
    .unwrap()
});

/// Counter for number of periodic garbage-collection (=GC) events that happen, regardless of
/// how many txns were actually cleaned up in this GC event
pub static CORE_MEMPOOL_GC_EVENT_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_core_mempool_gc_event_count",
        "Number of times the periodic garbage-collection event occurs, regardless of how many txns were actually removed",
        &["type"])
       .unwrap()
});

/// Counter tracking time for how long a transaction stayed in core-mempool before being garbage-collected
pub static CORE_MEMPOOL_GC_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_core_mempool_gc_latency",
        "How long a transaction stayed in core mempool before garbage-collected",
        &["type", "status"]
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
static MEMPOOL_SERVICE_TXNS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_mempool_service_transactions",
        "Number of transactions handled in one request/response between mempool and consensus/state sync",
        &["type"]
    )
        .unwrap()
});

pub fn mempool_service_transactions(label: &'static str, num: usize) {
    MEMPOOL_SERVICE_TXNS
        .with_label_values(&[label])
        .observe(num as f64)
}

/// Counter for tracking latency of mempool processing requests from consensus/state sync
/// A 'fail' result means the mempool's callback response to consensus/state sync failed.
static MEMPOOL_SERVICE_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_mempool_service_latency_ms",
        "Latency of mempool processing request from consensus/state sync",
        &["type", "result"]
    )
    .unwrap()
});

pub fn mempool_service_latency(label: &'static str, result: &str, duration: Duration) {
    MEMPOOL_SERVICE_LATENCY
        .with_label_values(&[label, result])
        .observe(duration.as_secs_f64());
}

/// Counter for types of network messages received by shared mempool
static SHARED_MEMPOOL_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_shared_mempool_events",
        "Number of network events received by shared mempool",
        &["event"] // type of event: "new_peer", "lost_peer", "message"
    )
    .unwrap()
});

pub fn shared_mempool_event_inc(event: &'static str) {
    SHARED_MEMPOOL_EVENTS.with_label_values(&[event]).inc();
}

/// Counter for tracking e2e latency for mempool to process txn submission requests from clients and peers
static PROCESS_TXN_SUBMISSION_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_shared_mempool_request_latency",
        "Latency of mempool processing txn submission requests",
        &["network", "sender"] // sender of txn(s)
    )
    .unwrap()
});

pub fn process_txn_submit_latency_timer(network: &str, sender: &str) -> HistogramTimer {
    PROCESS_TXN_SUBMISSION_LATENCY
        .with_label_values(&[network, sender])
        .start_timer()
}

/// Tracks latency of different stages of txn processing (e.g. vm validation, storage read)
pub static PROCESS_TXN_BREAKDOWN_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_mempool_process_txn_breakdown_latency",
        "Latency of different stages of processing txns in mempool",
        &["portion"]
    )
    .unwrap()
});

/// Counter for tracking latency for mempool to broadcast to a peer
pub static SHARED_MEMPOOL_BROADCAST_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_broadcast_latency",
        "Latency of mempool executing broadcast to another peer",
        &["network", "recipient"]
    )
    .unwrap()
});

/// Counter for tracking roundtrip-time from sending a broadcast to receiving ACK for that broadcast
pub static SHARED_MEMPOOL_BROADCAST_RTT: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_shared_mempool_broadcast_roundtrip_latency",
        "Time elapsed between sending a broadcast and receiving an ACK for that broadcast",
        &["network", "recipient"]
    )
    .unwrap()
});

/// Counter tracking number of mempool broadcasts that have not been ACK'ed for
static SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_shared_mempool_pending_broadcasts_count",
        "Number of mempool broadcasts not ACK'ed for yet",
        &["network", "recipient"]
    )
    .unwrap()
});

pub fn shared_mempool_pending_broadcasts(peer: &PeerNetworkId) -> IntGauge {
    SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT.with_label_values(&[
        peer.raw_network_id().as_str(),
        peer.peer_id().short_str().as_str(),
    ])
}

static SHARED_MEMPOOL_TRANSACTIONS_PROCESSED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_shared_mempool_transactions_processed",
        "Number of transactions received and handled by shared mempool",
        &[
            "status", // state of transaction processing: "received", "success", status code from failed txn processing
            "network", // state of transaction processing: "received", "success", status code from failed txn processing
            "sender"   // sender of the txns
        ]
    )
    .unwrap()
});

pub fn shared_mempool_transactions_processed_inc(status: &str, network: &str, sender: &str) {
    SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
        .with_label_values(&[status, network, sender])
        .inc();
}

/// Counter for number of transactions in each mempool broadcast sent
pub static SHARED_MEMPOOL_TRANSACTION_BROADCAST_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_shared_mempool_transaction_broadcast",
        "Number of transactions in each mempool broadcast sent",
        &["network", "recipient"]
    )
    .unwrap()
});

pub static SHARED_MEMPOOL_BROADCAST_TYPE_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_shared_mempool_rebroadcast_count",
        "Number of various types of broadcasts executed by shared mempool",
        &["network", "recipient", "type"]
    )
    .unwrap()
});

static SHARED_MEMPOOL_ACK_TYPE_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_shared_mempool_ack_count",
        "Number of various types of ACKs sent/received by shared mempool",
        &["network", "recipient", "direction", "type"]
    )
    .unwrap()
});

pub fn shared_mempool_ack_inc(peer: &PeerNetworkId, direction: &str, label: &'static str) {
    SHARED_MEMPOOL_ACK_TYPE_COUNT
        .with_label_values(&[
            peer.raw_network_id().as_str(),
            peer.peer_id().short_str().as_str(),
            direction,
            label,
        ])
        .inc();
}

static TASK_SPAWN_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_mempool_bounded_executor_spawn_latency",
        "Time it takes for mempool's coordinator to spawn async tasks",
        &["task", "stage"]
    )
    .unwrap()
});

pub fn task_spawn_latency_timer(task: &'static str, stage: &'static str) -> HistogramTimer {
    TASK_SPAWN_LATENCY
        .with_label_values(&[task, stage])
        .start_timer()
}

pub static CORE_MEMPOOL_INVARIANT_VIOLATION_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_mempool_core_mempool_invariant_violated_count",
        "Number of times a core mempool invariant was violated"
    )
    .unwrap()
});

pub static VM_RECONFIG_UPDATE_FAIL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_mempool_vm_reconfig_update_fail_count",
        "Number of times mempool's VM reconfig update failed"
    )
    .unwrap()
});

/// Counter for failed Diem network sends
static NETWORK_SEND_FAIL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_mempool_network_send_fail_count",
        "Number of times mempool network send failure occurs",
        &["type"]
    )
    .unwrap()
});

pub fn network_send_fail_inc(label: &'static str) {
    NETWORK_SEND_FAIL.with_label_values(&[label]).inc();
}

static UNEXPECTED_NETWORK_MSG_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_mempool_unexpected_network_count",
        "Number of unexpected network msgs received",
        &["network", "peer"]
    )
    .unwrap()
});

pub fn unexpected_msg_count_inc(network_id: &NetworkId, peer_id: &PeerId) {
    UNEXPECTED_NETWORK_MSG_COUNT
        .with_label_values(&[network_id.as_str(), peer_id.short_str().as_str()])
        .inc();
}

/// Counter for failed callback response to JSON RPC
pub static CLIENT_CALLBACK_FAIL: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_mempool_json_rpc_callback_fail_count",
        "Number of times callback to JSON RPC failed in mempool"
    )
    .unwrap()
});

/// Counter for how many ACKs were received with an invalid request_id that this node's mempool
/// did not send
static INVALID_ACK_RECEIVED_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_mempool_unrecognized_ack_received_count",
        "Number of ACK messages received with an invalid request_id that this node's mempool did not send",
        &["network", "sender", "type"]
    )
        .unwrap()
});

pub fn invalid_ack_inc(peer: &PeerNetworkId, label: &'static str) {
    INVALID_ACK_RECEIVED_COUNT
        .with_label_values(&[
            peer.raw_network_id().as_str(),
            peer.peer_id().short_str().as_str(),
            label,
        ])
        .inc();
}

/// Counter for number of times a DB read resulted in error
pub static DB_ERROR: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_mempool_db_error_count",
        "Number of times a DB read error was encountered in mempool"
    )
    .unwrap()
});

/// Gauge for the preference ranking of the current chosen upstream network
/// to broadcast to
static UPSTREAM_NETWORK: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_mempool_upstream_network",
        "The preference of the network mempool is broadcasting to"
    )
    .unwrap()
});

pub fn upstream_network(network: usize) {
    UPSTREAM_NETWORK.set(network as i64)
}

/// Counter for the current number of active upstream peers mempool can
/// broadcast to, summed across each of its networks
static ACTIVE_UPSTREAM_PEERS_COUNT: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_mempool_active_upstream_peers_count",
        "Number of active upstream peers for the node of this mempool",
        &["network"]
    )
    .unwrap()
});

pub fn active_upstream_peers(network_id: &NetworkId) -> IntGauge {
    ACTIVE_UPSTREAM_PEERS_COUNT.with_label_values(&[network_id.as_str()])
}

/// Duration of each run of the event loop.
pub static MAIN_LOOP: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "diem_mempool_main_loop",
            "Duration of the each run of the event loop"
        )
        .unwrap(),
    )
});
