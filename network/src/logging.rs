// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! This module is to contain all networking logging information.
//!
//! ```
//! use diem_config::network_id::NetworkContext;
//! use diem_logger::info;
//! use diem_types::{PeerId, network_address::NetworkAddress};
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
use diem_config::network_id::NetworkContext;
use diem_logger::Schema;
use diem_types::{network_address::NetworkAddress, PeerId};
use netcore::transport::ConnectionOrigin;

#[derive(Schema)]
pub struct NetworkSchema<'a> {
    connection_id: Option<&'a ConnectionId>,
    #[schema(display)]
    connection_origin: Option<&'a ConnectionOrigin>,
    #[schema(display)]
    discovery_source: Option<&'a DiscoverySource>,
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
            network_address: None,
            network_context,
            remote_peer: None,
        }
    }

    pub fn connection_metadata(self, metadata: &'a ConnectionMetadata) -> Self {
        self.connection_id(&metadata.connection_id)
            .connection_origin(&metadata.origin)
            .remote_peer(&metadata.remote_peer_id)
    }

    pub fn connection_metadata_with_address(self, metadata: &'a ConnectionMetadata) -> Self {
        self.connection_id(&metadata.connection_id)
            .connection_origin(&metadata.origin)
            .remote_peer(&metadata.remote_peer_id)
            .network_address(&metadata.addr)
    }
}
