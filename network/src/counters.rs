// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use libra_metrics::{Histogram, IntCounter, IntGauge, OpMetrics};
use prometheus::IntGaugeVec;

lazy_static::lazy_static! {
    pub static ref NETWORK_PEERS: IntGaugeVec = register_int_gauge_vec!(
        // metric name
        "libra_network_peers",
        // metric description
        "Libra network peers counter",
        // metric labels (dimensions)
        &["state", "role"]
    ).unwrap();
}

lazy_static::lazy_static! {
    pub static ref OP_COUNTERS: OpMetrics = OpMetrics::new_and_registered("network");
}

lazy_static::lazy_static! {
    /// Counter of currently connected peers
    pub static ref CONNECTED_PEERS: IntGauge = OP_COUNTERS.gauge("connected_peers");

    /// Counter of rpc requests sent
    pub static ref RPC_REQUESTS_SENT: IntCounter = OP_COUNTERS.counter("rpc_requests_sent");

    /// Counter of rpc request bytes sent
    pub static ref RPC_REQUEST_BYTES_SENT: IntCounter = OP_COUNTERS.counter("rpc_request_bytes_sent");

    /// Counter of rpc requests failed
    pub static ref RPC_REQUESTS_FAILED: IntCounter = OP_COUNTERS.counter("rpc_requests_failed");

    /// Counter of rpc requests cancelled
    pub static ref RPC_REQUESTS_CANCELLED: IntCounter = OP_COUNTERS.counter("rpc_requests_cancelled");

    /// Counter of rpc requests received
    pub static ref RPC_REQUESTS_RECEIVED: IntCounter = OP_COUNTERS.counter("rpc_requests_received");

    /// Counter of rpc responses sent
    pub static ref RPC_RESPONSES_SENT: IntCounter = OP_COUNTERS.counter("rpc_responses_sent");

    /// Counter of rpc response bytes sent
    pub static ref RPC_RESPONSE_BYTES_SENT: IntCounter = OP_COUNTERS.counter("rpc_response_bytes_sent");

    /// Counter of rpc responses failed
    pub static ref RPC_RESPONSES_FAILED: IntCounter = OP_COUNTERS.counter("rpc_responses_failed");

    /// Histogram of rpc latency
    pub static ref RPC_LATENCY: Histogram = OP_COUNTERS.histogram("rpc_latency");

    /// Counter of messages sent via the direct send protocol
    pub static ref DIRECT_SEND_MESSAGES_SENT: IntCounter = OP_COUNTERS.counter("direct_send_messages_sent");

    /// Counter of bytes sent via the direct send protocol
    pub static ref DIRECT_SEND_BYTES_SENT: IntCounter = OP_COUNTERS.counter("direct_send_bytes_sent");

    /// Counter of messages dropped via the direct send protocol
    pub static ref DIRECT_SEND_MESSAGES_DROPPED: IntCounter = OP_COUNTERS.counter("direct_send_messages_dropped");

    /// Counter of messages received via the direct send protocol
    pub static ref DIRECT_SEND_MESSAGES_RECEIVED: IntCounter = OP_COUNTERS.counter("direct_send_messages_received");

    /// Counter of bytes received via the direct send protocol
    pub static ref DIRECT_SEND_BYTES_RECEIVED: IntCounter = OP_COUNTERS.counter("direct_send_bytes_received");

    ///
    /// Channel Counters
    ///

    /// Counter of pending requests in Network Provider
    pub static ref PENDING_NETWORK_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_network_requests");

    /// Counter of pending network events to Mempool
    pub static ref PENDING_MEMPOOL_NETWORK_EVENTS: IntGauge = OP_COUNTERS.gauge("pending_mempool_network_events");

    /// Counter of pending network events to Consensus
    pub static ref PENDING_CONSENSUS_NETWORK_EVENTS: IntGauge = OP_COUNTERS.gauge("pending_consensus_network_events");

    /// Counter of pending network events to State Synchronizer
    pub static ref PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS: IntGauge = OP_COUNTERS.gauge("pending_state_sync_network_events");

    /// Counter of pending network events to Admission Control
    pub static ref PENDING_ADMISSION_CONTROL_NETWORK_EVENTS: IntGauge = OP_COUNTERS.gauge("pending_admission_control_network_events");

    /// Counter of pending requests in Peer Manager
    pub static ref PENDING_PEER_MANAGER_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_requests");

    /// Counter of pending Peer Manager notifications in Network Provider
    pub static ref PENDING_PEER_MANAGER_NET_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_net_notifications");

    /// Counter of pending requests in Direct Send
    pub static ref PENDING_DIRECT_SEND_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_direct_send_requests");

    /// Counter of pending Direct Send notifications to Network Provider
    pub static ref PENDING_DIRECT_SEND_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_direct_send_notifications");

    /// Counter of pending requests in Connectivity Manager
    pub static ref PENDING_CONNECTIVITY_MANAGER_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_connectivity_manager_requests");

    /// Counter of pending requests in RPC
    pub static ref PENDING_RPC_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_rpc_requests");

    /// Counter of pending RPC notifications to Network Provider
    pub static ref PENDING_RPC_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_rpc_notifications");

    /// Counter of pending Peer Manager notifications to Direct Send
    pub static ref PENDING_PEER_MANAGER_DIRECT_SEND_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_direct_send_notifications");

    /// Counter of pending Peer Manager notifications to RPC
    pub static ref PENDING_PEER_MANAGER_RPC_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_rpc_notifications");

    /// Counter of pending Peer Manager notifications to Discovery
    pub static ref PENDING_PEER_MANAGER_DISCOVERY_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_discovery_notifications");

    /// Counter of pending Peer Manager notifications to Ping
    pub static ref PENDING_PEER_MANAGER_PING_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_ping_notifications");

    /// Counter of pending Peer Manager notifications to Connectivity Manager
    pub static ref PENDING_PEER_MANAGER_CONNECTIVITY_MANAGER_NOTIFICATIONS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_connectivity_manager_notifications");

    /// Counter of pending internal events in Peer Manager
    pub static ref PENDING_PEER_MANAGER_INTERNAL_EVENTS: IntGauge = OP_COUNTERS.gauge("pending_peer_manager_internal_events");

    /// Counter of pending dial requests in Peer Manager
    pub static ref PENDING_PEER_MANAGER_DIAL_REQUESTS: IntGauge  = OP_COUNTERS.gauge("pending_peer_manager_dial_requests");

    /// Counter of pending requests for each remote peer
    pub static ref PENDING_PEER_REQUESTS: &'static str = "pending_peer_requests";

    /// Counter of pending outbound messages in Direct Send for each remote peer
    pub static ref PENDING_DIRECT_SEND_OUTBOUND_MESSAGES: &'static str = "pending_direct_send_outbound_messages";
}
