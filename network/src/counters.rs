// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{Histogram, IntGauge, OpMetrics};
use once_cell::sync::Lazy;
use prometheus::{HistogramVec, IntCounterVec, IntGaugeVec};

pub static LIBRA_NETWORK_PEERS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "libra_network_peers",
        // metric description
        "Libra network peers counter",
        // metric labels (dimensions)
        &["role_type", "state"]
    )
    .unwrap()
});

pub static LIBRA_NETWORK_DISCOVERY_NOTES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "libra_network_discovery_notes",
        // metric description
        "Libra network discovery notes",
        // metric labels (dimensions)
        &["role_type"]
    )
    .unwrap()
});

pub static LIBRA_NETWORK_RPC_MESSAGES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_rpc_messages",
        "Libra network rpc messages counter",
        &["type", "state"]
    )
    .unwrap()
});

pub static LIBRA_NETWORK_RPC_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_network_rpc_bytes",
        "Libra network rpc bytes histogram",
        &["type", "state"]
    )
    .unwrap()
});

pub static LIBRA_NETWORK_RPC_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_network_rpc_latency_seconds",
        "Libra network rpc latency histogram"
    )
    .unwrap()
});

pub static LIBRA_NETWORK_DIRECT_SEND_MESSAGES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_direct_send_messages",
        "Libra network direct send messages counter",
        &["state"]
    )
    .unwrap()
});

pub static LIBRA_NETWORK_DIRECT_SEND_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_network_direct_send_bytes",
        "Libra network direct send bytes histogram",
        &["state"]
    )
    .unwrap()
});

pub static OP_COUNTERS: Lazy<OpMetrics> = Lazy::new(|| OpMetrics::new_and_registered("network"));

///
/// Channel Counters
///

/// Counter of pending requests in Network Provider
pub static PENDING_NETWORK_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_network_requests"));

/// Counter of pending network events to Mempool
pub static PENDING_MEMPOOL_NETWORK_EVENTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_mempool_network_events"));

/// Counter of pending network events to Consensus
pub static PENDING_CONSENSUS_NETWORK_EVENTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_consensus_network_events"));

/// Counter of pending network events to State Synchronizer
pub static PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_state_sync_network_events"));

/// Counter of pending network events to Admission Control
pub static PENDING_ADMISSION_CONTROL_NETWORK_EVENTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_admission_control_network_events"));

/// Counter of pending network events to Health Checker.
pub static PENDING_HEALTH_CHECKER_NETWORK_EVENTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_health_checker_network_events"));

/// Counter of pending network events to Discovery.
pub static PENDING_DISCOVERY_NETWORK_EVENTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_discovery_network_events"));

/// Counter of pending requests in Peer Manager
pub static PENDING_PEER_MANAGER_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_manager_requests"));

/// Counter of pending Peer Manager notifications in Network Provider
pub static PENDING_PEER_MANAGER_NET_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_manager_net_notifications"));

/// Counter of pending requests in Direct Send
pub static PENDING_DIRECT_SEND_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_direct_send_requests"));

/// Counter of pending Direct Send notifications to Network Provider
pub static PENDING_DIRECT_SEND_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_direct_send_notifications"));

/// Counter of pending requests in Connectivity Manager
pub static PENDING_CONNECTIVITY_MANAGER_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_connectivity_manager_requests"));

/// Counter of pending requests in RPC
pub static PENDING_RPC_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_rpc_requests"));

/// Counter of pending RPC notifications to Network Provider
pub static PENDING_RPC_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_rpc_notifications"));

/// Counter of pending Peer Manager notifications to Direct Send
pub static PENDING_PEER_MANAGER_DIRECT_SEND_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_manager_direct_send_notifications"));

/// Counter of pending Peer Manager notifications to RPC
pub static PENDING_PEER_MANAGER_RPC_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_manager_rpc_notifications"));

/// Counter of pending Peer Manager notifications to Connectivity Manager
pub static PENDING_PEER_MANAGER_CONNECTIVITY_MANAGER_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_manager_connectivity_manager_notifications"));

/// Counter of pending Peer events to PeerManager.
pub static PENDING_PEER_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_notifications"));

/// Counter of pending Connection Handler notifications to PeerManager.
pub static PENDING_CONNECTION_HANDLER_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_connection_handler_notifications"));

/// Counter of pending dial requests in Peer Manager
pub static PENDING_PEER_MANAGER_DIAL_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("pending_peer_manager_dial_requests"));

/// Counter of pending requests for each remote peer
pub static PENDING_PEER_REQUESTS: &str = "pending_peer_requests";

/// Counter of pending outbound messages in Direct Send for each remote peer
pub static PENDING_DIRECT_SEND_OUTBOUND_MESSAGES: &str = "pending_direct_send_outbound_messages";
