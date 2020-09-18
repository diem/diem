// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::network_id::NetworkContext;
use libra_metrics::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, Histogram,
    HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, OpMetrics,
};
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;
use once_cell::sync::Lazy;

// some type labels
pub const REQUEST_LABEL: &str = "request";
pub const RESPONSE_LABEL: &str = "response";

// some state labels
pub const CANCELED_LABEL: &str = "canceled";
pub const DECLINED_LABEL: &str = "declined";
pub const RECEIVED_LABEL: &str = "received";
pub const SENT_LABEL: &str = "sent";
pub const SUCCEEDED_LABEL: &str = "succeeded";
pub const FAILED_LABEL: &str = "failed";

pub static LIBRA_NETWORK_PEERS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_network_peers",
        "Number of peers, and their associated state",
        &["role_type", "state"]
    )
    .unwrap()
});

pub static LIBRA_CONNECTIONS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_connections",
        "Number of current connections and their direction",
        &["role_type", "network_id", "peer_id", "direction"]
    )
    .unwrap()
});

pub fn connections(network_context: &NetworkContext, origin: ConnectionOrigin) -> IntGauge {
    LIBRA_CONNECTIONS.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        origin.as_str(),
    ])
}

pub static LIBRA_NETWORK_PEER_CONNECTED: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_network_peer_connected",
        "Indicates if we are connected to a particular peer",
        &["role_type", "network_id", "peer_id", "remote_peer_id"]
    )
    .unwrap()
});

pub fn peer_connected(network_context: &NetworkContext, remote_peer_id: &PeerId, v: i64) {
    if network_context.role().is_validator() {
        LIBRA_NETWORK_PEER_CONNECTED
            .with_label_values(&[
                network_context.role().as_str(),
                network_context.network_id().as_str(),
                network_context.peer_id().short_str().as_str(),
                remote_peer_id.short_str().as_str(),
            ])
            .set(v)
    }
}

pub static LIBRA_NETWORK_PENDING_CONNECTION_UPGRADES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_network_pending_connection_upgrades",
        "Number of concurrent inbound or outbound connections we're currently negotiating",
        &["role_type", "network_id", "peer_id", "direction"]
    )
    .unwrap()
});

pub fn pending_connection_upgrades(
    network_context: &NetworkContext,
    direction: ConnectionOrigin,
) -> IntGauge {
    LIBRA_NETWORK_PENDING_CONNECTION_UPGRADES.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        direction.as_str(),
    ])
}

pub static LIBRA_NETWORK_CONNECTION_UPGRADE_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_network_connection_upgrade_time_seconds",
        "Time to complete a new inbound or outbound connection upgrade",
        &["role_type", "network_id", "peer_id", "direction", "state"]
    )
    .unwrap()
});

pub fn connection_upgrade_time(
    network_context: &NetworkContext,
    direction: ConnectionOrigin,
    state: &'static str,
) -> Histogram {
    LIBRA_NETWORK_CONNECTION_UPGRADE_TIME.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        direction.as_str(),
        state,
    ])
}

pub static LIBRA_NETWORK_DISCOVERY_NOTES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "libra_network_discovery_notes",
        "Libra network discovery notes",
        &["role_type"]
    )
    .unwrap()
});

pub static LIBRA_NETWORK_RPC_MESSAGES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_rpc_messages",
        "Number of RPC messages",
        &["role_type", "network_id", "peer_id", "type", "state"]
    )
    .unwrap()
});

pub fn rpc_messages(
    network_context: &NetworkContext,
    type_label: &'static str,
    state_label: &'static str,
) -> IntCounter {
    LIBRA_NETWORK_RPC_MESSAGES.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        type_label,
        state_label,
    ])
}

pub static LIBRA_NETWORK_RPC_BYTES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_rpc_bytes",
        "Number of RPC bytes transferred",
        &["role_type", "network_id", "peer_id", "type", "state"]
    )
    .unwrap()
});

pub fn rpc_bytes(
    network_context: &NetworkContext,
    type_label: &'static str,
    state_label: &'static str,
) -> IntCounter {
    LIBRA_NETWORK_RPC_BYTES.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        type_label,
        state_label,
    ])
}

// TODO(philiphayes): specify that this is outbound rpc only
// TODO(philiphayes): somehow get per-peer latency metrics without using a
// separate peer_id label ==> cardinality explosion.

pub static LIBRA_NETWORK_RPC_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "libra_network_rpc_latency_seconds",
        "RPC request latency in seconds",
        &["role_type", "network_id", "peer_id"]
    )
    .unwrap()
});

pub fn rpc_latency(network_context: &NetworkContext) -> Histogram {
    LIBRA_NETWORK_RPC_LATENCY.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
    ])
}

pub static LIBRA_NETWORK_DIRECT_SEND_MESSAGES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_direct_send_messages",
        "Number of direct send messages",
        &["role_type", "network_id", "peer_id", "state"]
    )
    .unwrap()
});

pub fn direct_send_messages(
    network_context: &NetworkContext,
    state_label: &'static str,
) -> IntCounter {
    LIBRA_NETWORK_DIRECT_SEND_MESSAGES.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        state_label,
    ])
}

pub static LIBRA_NETWORK_DIRECT_SEND_BYTES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_direct_send_bytes",
        "Number of direct send bytes transferred",
        &["role_type", "network_id", "peer_id", "state"]
    )
    .unwrap()
});

pub fn direct_send_bytes(
    network_context: &NetworkContext,
    state_label: &'static str,
) -> IntCounter {
    LIBRA_NETWORK_DIRECT_SEND_BYTES.with_label_values(&[
        network_context.role().as_str(),
        network_context.network_id().as_str(),
        network_context.peer_id().short_str().as_str(),
        state_label,
    ])
}

/// Counters(queued,dequeued,dropped) related to inbound network notifications for RPCs and
/// DirectSends.
pub static PENDING_NETWORK_NOTIFICATIONS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_pending_network_notifications",
        "Number of pending inbound network notifications by state",
        &["state"]
    )
    .unwrap()
});

/// Counter of pending requests in Network Provider
pub static PENDING_NETWORK_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_pending_requests",
        "Number of pending outbound network requests by state",
        &["state"]
    )
    .unwrap()
});

/// Counter of pending network events to Health Checker.
pub static PENDING_HEALTH_CHECKER_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_pending_health_check_events",
        "Number of pending health check events by state",
        &["state"]
    )
    .unwrap()
});

/// Counter of pending network events to Discovery.
pub static PENDING_DISCOVERY_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_pending_discovery_events",
        "Number of pending discovery events by state",
        &["state"]
    )
    .unwrap()
});

/// Counter of pending requests in Peer Manager
pub static PENDING_PEER_MANAGER_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_network_pending_peer_manager_requests",
        "Number of pending peer manager requests by state",
        &["state"]
    )
    .unwrap()
});

pub static OP_COUNTERS: Lazy<OpMetrics> = Lazy::new(|| OpMetrics::new_and_registered("network"));

///
/// Channel Counters
///

/// Counter of pending requests in Connectivity Manager
pub static PENDING_CONNECTIVITY_MANAGER_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("libra_network_pending_connectivity_manager_requests"));

/// Counter of pending Connection Handler notifications to PeerManager.
pub static PENDING_CONNECTION_HANDLER_NOTIFICATIONS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("libra_network_pending_connection_handler_notifications"));

/// Counter of pending dial requests in Peer Manager
pub static PENDING_PEER_MANAGER_DIAL_REQUESTS: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("libra_network_pending_peer_manager_dial_requests"));

/// Counter of messages pending in queue to be sent out on the wire.
pub static PENDING_WIRE_MESSAGES: Lazy<IntGauge> =
    Lazy::new(|| OP_COUNTERS.gauge("libra_network_pending_wire_messages"));

/// Counter of pending requests in Direct Send
pub static PENDING_DIRECT_SEND_REQUESTS: &str = "libra_network_pending_direct_send_requests";

/// Counter of pending Direct Send notifications to Network Provider
pub static PENDING_DIRECT_SEND_NOTIFICATIONS: &str =
    "libra_network_pending_direct_send_notifications";

/// Counter of pending requests in RPC
pub static PENDING_RPC_REQUESTS: &str = "libra_network_pending_rpc_requests";

/// Counter of pending RPC notifications to Network Provider
pub static PENDING_RPC_NOTIFICATIONS: &str = "libra_network_pending_rpc_notifications";

/// Counter of pending requests for each remote peer
pub static PENDING_PEER_REQUESTS: &str = "libra_network_pending_peer_requests";

/// Counter of pending RPC events from Peer to Rpc actor.
pub static PENDING_PEER_RPC_NOTIFICATIONS: &str = "libra_network_pending_peer_rpc_notifications";

/// Counter of pending DirectSend events from Peer to DirectSend actor..
pub static PENDING_PEER_DIRECT_SEND_NOTIFICATIONS: &str =
    "libra_network_pending_peer_direct_send_notifications";

/// Counter of pending connection notifications from Peer to NetworkProvider.
pub static PENDING_PEER_NETWORK_NOTIFICATIONS: &str = "libra_network_pending_peer_notifications";
