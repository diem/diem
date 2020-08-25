// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! This module is to contain all networking logging information.
//!
//! ```
//! use libra_config::network_id::NetworkContext;
//! use libra_logger::prelude::*;
//! use network::logging::*;
//!
//! sl_info!(
//!   network_log(network_events::CONNECTIVITY_MANAGER_LOOP, &NetworkContext::mock())
//!     .data(network_events::TYPE, network_events::START)
//!     .field(network_events::EVENT_ID, &5)
//! );
//! ```
//!

use libra_config::network_id::NetworkContext;
use libra_logger::StructuredLogEntry;

/// A helper function to cut down on a bunch of repeated network struct log code
pub fn network_log(label: &'static str, network_context: &NetworkContext) -> StructuredLogEntry {
    StructuredLogEntry::new("network", label)
        .field(network_events::NETWORK_CONTEXT, network_context)
}

/// This module is to ensure no conflicts with already existing constants
pub mod network_events {
    use crate::{
        connectivity_manager::DiscoverySource,
        peer_manager::{ConnectionNotification, ConnectionRequest, PeerManagerRequest},
        transport::ConnectionMetadata,
        ConnectivityRequest,
    };
    use libra_config::network_id::NetworkContext;
    use libra_logger::LoggingField;
    use libra_network_address::NetworkAddress;
    use libra_types::PeerId;

    /// Labels
    pub const CONNECTIVITY_MANAGER_LOOP: &str = "connectivity_manager_loop";
    pub const NOISE_UPGRADE: &str = "noise_upgrade";
    pub const PEER_MANAGER_LOOP: &str = "peer_manager_loop";
    pub const TRANSPORT_EVENT: &str = "transport_event";

    /// Common terms
    pub const TYPE: &str = "type";
    pub const START: &str = "start";
    pub const TERMINATION: &str = "termination";
    pub const EVENT: &str = "event";

    /// Specific fields for logging
    pub const NETWORK_CONTEXT: &LoggingField<&NetworkContext> =
        &LoggingField::new("network_context");
    pub const EVENT_ID: &LoggingField<&u32> = &LoggingField::new("event_id");
    pub const REMOTE_PEER: &LoggingField<&PeerId> = &LoggingField::new("remote_peer");
    pub const CONNECTION_NOTIFICATION: &LoggingField<&ConnectionNotification> =
        &LoggingField::new("conn_notification");
    pub const CONNECTIVITY_REQUEST: &LoggingField<&ConnectivityRequest> =
        &LoggingField::new("connectivity_request");
    pub const CONNECTION_REQUEST: &LoggingField<&ConnectionRequest> =
        &LoggingField::new("connection_request");
    pub const PEER_MANAGER_REQUEST: &LoggingField<&PeerManagerRequest> =
        &LoggingField::new("peer_manager_request");
    pub const NETWORK_ADDRESS: &LoggingField<&NetworkAddress> =
        &LoggingField::new("network_address");
    pub const DISCOVERY_SOURCE: &LoggingField<&DiscoverySource> =
        &LoggingField::new("discovery_source");
    pub const CONNECTION_METADATA: &LoggingField<&ConnectionMetadata> =
        &LoggingField::new("connection_metadata");
}
