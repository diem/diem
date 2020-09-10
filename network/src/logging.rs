// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! This module is to contain all networking logging information.
//!
//! ```
//! use libra_config::network_id::NetworkContext;
//! use libra_logger::info;
//! use libra_network_address::NetworkAddress;
//! use libra_types::PeerId;
//! use network::logging::NetworkSchema;
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

use crate::{
    connectivity_manager::DiscoverySource,
    transport::{ConnectionId, ConnectionMetadata},
};
use libra_config::network_id::NetworkContext;
use libra_logger::Schema;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;

#[derive(Schema)]
pub struct NetworkSchema<'a> {
    connection_id: Option<&'a ConnectionId>,
    #[schema(display)]
    connection_origin: Option<&'a ConnectionOrigin>,
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
            connection_id: None,
            connection_origin: None,
            discovery_source: None,
            error: None,
            network_address: None,
            network_context,
            remote_peer: None,
        }
    }

    pub fn connection_metadata(self, metadata: &'a ConnectionMetadata) -> Self {
        self.connection_id(&metadata.connection_id)
            .connection_origin(&metadata.origin)
            .network_address(&metadata.addr)
            .remote_peer(&metadata.peer_id)
    }

    pub fn debug_error<Err: std::fmt::Debug>(self, error: &Err) -> Self {
        self.error(format!("{:?}", error))
    }
}
