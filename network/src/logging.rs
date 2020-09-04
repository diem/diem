// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! This module is to contain all networking logging information.
//!
//! ```
//! use libra_config::network_id::NetworkContext;
//! use libra_logger::prelude::*;
//! use network::logging::*;
//! use network::connectivity_manager::DiscoverySource;
//! use futures::io::ErrorKind;
//! use libra_network_address::NetworkAddress;
//! use libra_types::PeerId;
//!
//! info!(
//!   NetworkSchema::new(&NetworkContext::mock())
//!     .remote_peer(&PeerId::random())
//!     .network_address(&NetworkAddress::mock()),
//!   field_name = "field",
//!   "Value is {} message",
//!   5
//! );
//! ```

use crate::connectivity_manager::DiscoverySource;
use libra_config::network_id::NetworkContext;
use libra_logger::{Schema, StructuredLogEntry};
use libra_network_address::NetworkAddress;
use libra_types::PeerId;

/// A helper function to cut down on a bunch of repeated network struct log code
pub fn network_log(label: &'static str, network_context: &NetworkContext) -> StructuredLogEntry {
    StructuredLogEntry::new_named("network", label)
        .field(network_events::NETWORK_CONTEXT, network_context)
}

#[derive(Schema)]
pub struct NetworkSchema<'a> {
    #[schema(display)]
    discovery_source: Option<&'a DiscoverySource>,
    error: Option<String>,
    #[schema(display)]
    network_address: Option<&'a NetworkAddress>,
    network_context: &'a NetworkContext,
    #[schema(display)]
    remote_peer: Option<&'a PeerId>,
}

impl<'a> NetworkSchema<'a> {
    pub fn new(network_context: &'a NetworkContext) -> Self {
        Self {
            discovery_source: None,
            error: None,
            network_address: None,
            network_context,
            remote_peer: None,
        }
    }
}

/// This module is to ensure no conflicts with already existing constants
pub mod network_events {
    use crate::{
        peer_manager::{ConnectionRequest, PeerManagerRequest},
        transport::ConnectionMetadata,
    };
    use libra_config::network_id::NetworkContext;
    use libra_logger::LoggingField;
    use libra_network_address::NetworkAddress;
    use libra_types::PeerId;

    /// Labels
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
    pub const CONNECTION_REQUEST: &LoggingField<&ConnectionRequest> =
        &LoggingField::new("connection_request");
    pub const PEER_MANAGER_REQUEST: &LoggingField<&PeerManagerRequest> =
        &LoggingField::new("peer_manager_request");
    pub const NETWORK_ADDRESS: &LoggingField<&NetworkAddress> =
        &LoggingField::new("network_address");
    pub const CONNECTION_METADATA: &LoggingField<&ConnectionMetadata> =
        &LoggingField::new("connection_metadata");
}
