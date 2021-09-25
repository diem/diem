// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    application::{
        interface::NetworkInterface,
        storage::{LockingBTreeMap, PeerMetadataStorage},
        types::{PeerError, PeerState},
    },
    transport::ConnectionMetadata,
};
use diem_types::PeerId;
use std::{collections::hash_map::Entry, sync::Arc};

/// Dummy network so we can test the interfaces
struct DummyNetworkInterface {
    peer_metadata_storage: Arc<PeerMetadataStorage>,
    app_data: LockingBTreeMap<PeerId, usize>,
}

impl NetworkInterface for DummyNetworkInterface {
    type Sender = ();
    type AppData = usize;

    fn peer_metadata_storage(&self) -> &PeerMetadataStorage {
        &self.peer_metadata_storage
    }

    fn sender(&self) -> Self::Sender {}

    fn insert_app_data(&self, peer_id: PeerId, data: Self::AppData) {
        self.app_data.insert(peer_id, data)
    }

    fn remove_app_data(&self, peer_id: &PeerId) {
        self.app_data.remove(peer_id)
    }

    fn read_app_data(&self, peer_id: &PeerId) -> Option<Self::AppData> {
        self.app_data.read(peer_id)
    }

    fn write_app_data<F: FnOnce(&mut Entry<PeerId, Self::AppData>) -> Result<(), PeerError>>(
        &self,
        peer_id: PeerId,
        modifier: F,
    ) -> Result<(), PeerError> {
        self.app_data.write(peer_id, modifier)
    }
}

#[test]
fn test_interface() {
    let peer_metadata_storage = Arc::new(PeerMetadataStorage::new());
    let interface = DummyNetworkInterface {
        peer_metadata_storage: peer_metadata_storage.clone(),
        app_data: LockingBTreeMap::new(),
    };
    let peer_1 = PeerId::random();
    let peer_2 = PeerId::random();
    assert_eq!(0, interface.peers().len());
    assert_eq!(0, interface.connected_peers().len());

    // Insert 2 connections, and we should have two active peers
    let connection_1 = ConnectionMetadata::mock(peer_1);
    let connection_2 = ConnectionMetadata::mock(peer_2);
    peer_metadata_storage.insert_connection(connection_1);
    peer_metadata_storage.insert_connection(connection_2.clone());
    assert_eq!(2, interface.peers().len());
    assert_eq!(2, interface.connected_peers().len());

    // Disconnecting / disconnected are not counted in active
    update_state(
        peer_metadata_storage.clone(),
        peer_1,
        PeerState::Disconnecting,
    );
    assert_eq!(2, interface.peers().len());
    assert_eq!(1, interface.connected_peers().len());

    // Removing a connection with a different connection id doesn't remove it from storage
    let different_connection_2 = ConnectionMetadata::mock(peer_2);
    peer_metadata_storage.remove_connection(&different_connection_2);
    assert_eq!(2, interface.peers().len());
    assert_eq!(1, interface.connected_peers().len());

    // Removing the same connection id removes it
    peer_metadata_storage.remove_connection(&connection_2);
    assert_eq!(1, interface.peers().len());
    assert_eq!(0, interface.connected_peers().len());
}

fn update_state(
    peer_metadata_storage: Arc<PeerMetadataStorage>,
    peer_id: PeerId,
    state: PeerState,
) {
    peer_metadata_storage
        .write(peer_id, |entry| match entry {
            Entry::Vacant(..) => Err(PeerError::NotFound),
            Entry::Occupied(inner) => {
                inner.get_mut().status = state;
                Ok(())
            }
        })
        .unwrap()
}
