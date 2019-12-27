// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use prometheus::{HistogramVec, IntCounterVec, IntGaugeVec};

lazy_static::lazy_static! {
    /// number of workers, by status
    pub static ref WORKERS: IntGaugeVec = register_int_gauge_vec!(
        // metric name
        "libra_mempool_workers_total",
        // metric description
        "Number of worker processes handling txn broadcasts",
        // metric_labels
        &[
            "status", // START, PAUSE, KILL
            "worker_peer_id"
        ]
    ).unwrap();

    /// number of new_peer / lost peer
    pub static ref PEER_NOTIFICATION: IntCounterVec = register_int_counter_vec!(
        "libra_mempool_peer_notifications_total",
        "Number of peer notifications that shared mempool receives",
        &["type", "notification_sender_peer_id"]
    ).unwrap();

    /// time between getting an RpcRequest and actually spawning it
    pub static ref RPC_PROCESS_SPAWN_TIME: HistogramVec = register_histogram_vec!(
        "libra_mempool_rpc_process_spawn_time_s",
        "Histogram of time it takes for an process to be spawned",
        &[
            "from_peer"
        ]
    ).unwrap();

    /// time needed to handle inbound RPC request
    pub static ref TXN_SUBMISSION_PROCESSING_TIME: HistogramVec = register_histogram_vec!(
        "libra_mempool_txn_submission_processing_time_s",
        "Histogram of time it takes for shared mempool to process an incoming txn broadcast",
        &[
            "sender_peer_id",
        ]
    ).unwrap();

    // outbound RPC call duration / timeouts

    // callback duration
    pub static ref OUTBOUND_TXN_BROADCAST_TIME: HistogramVec = register_histogram_vec!(
        "libra_mempool_outbound_txn_broadcast_time_duration_s",
        "Histogram of time it takes for ",
        &[
            "to_peer"
        ]
    ).unwrap();

    // callback timeouts
    pub static ref TIMEOUT: IntGaugeVec = register_int_gauge_vec!(
        "libra_mempool_rpc_timeouts_total",
        "Number of RPC timeouts",
        &[
            "other_peer", // peer id of rpc
            "direction", // callback, outbound
        ]
    ).unwrap();

    pub static ref SUBMIT_TXNS_MEMPOOL_TIME_BREAKDOWN: HistogramVec = register_histogram_vec!(
        "libra_mempool_process_txns_time_breakdown_ms",
        "Histogram of the time breakdown of submitting transactions to mempool",
        &[
            "sender_peer_id",
            "activity", // convert_proto, vm_validation, mempool_lock, add_txn, callback
        ]
    ).unwrap();

    pub static ref BROADCAST_TXNS_TIME_BREAKDOWN: HistogramVec = register_histogram_vec!(
        "libra_mempool_broadcast_txns_time_breakdown_ms",
        "Histogram of the time breakdown of broadcasting transactions to mempool",
        &[
            "sender_peer_id",
            "activity", // mempool_lock, network_send, master_push
        ]
    ).unwrap();
}
