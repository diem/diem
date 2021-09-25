// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::application::{
    storage::PeerMetadataStorage,
    types::{PeerError, PeerInfo, PeerState},
};
use async_trait::async_trait;
use diem_types::PeerId;
use std::collections::{hash_map::Entry, BTreeMap};

/// A generic `NetworkInterface` for applications to connect to networking
///
/// Each application would implement their own `NetworkInterface`.  This would hold `AppData` specific
/// to the application as well as a specific `Sender` for cloning across threads and sending requests.
#[async_trait]
pub trait NetworkInterface {
    type Sender;
    type AppData;

    /// Provides the `PeerMetadataStorage` for other functions.  Not expected to be used externally.
    fn peer_metadata_storage(&self) -> &PeerMetadataStorage;

    /// Give a copy of the sender for the network
    fn sender(&self) -> Self::Sender;

    /// Retrieve only connected peers
    fn connected_peers(&self) -> BTreeMap<PeerId, PeerInfo> {
        self.filtered_peers(|(_, peer_info)| peer_info.status == PeerState::Connected)
    }

    /// Filter peers with according `filter`
    fn filtered_peers<F: FnMut(&(&PeerId, &PeerInfo)) -> bool>(
        &self,
        filter: F,
    ) -> BTreeMap<PeerId, PeerInfo> {
        self.peer_metadata_storage().read_filtered(filter)
    }

    /// Retrieve PeerInfo for the node
    fn peers(&self) -> BTreeMap<PeerId, PeerInfo> {
        self.peer_metadata_storage().read_all()
    }

    /// Insert application specific data
    fn insert_app_data(&self, peer_id: PeerId, data: Self::AppData);

    /// Removes application specific data
    fn remove_app_data(&self, peer_id: &PeerId);

    /// Read application specific data
    fn read_app_data(&self, peer_id: &PeerId) -> Option<Self::AppData>;

    /// Write application specific data, allows for read before write operations
    fn write_app_data<F: FnOnce(&mut Entry<PeerId, Self::AppData>) -> Result<(), PeerError>>(
        &self,
        peer_id: PeerId,
        modifier: F,
    ) -> Result<(), PeerError>;
}
