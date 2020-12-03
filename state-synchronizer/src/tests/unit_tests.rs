// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::request_manager::{PeerScoreUpdateType, RequestManager};
use diem_config::config::{PeerNetworkId, UpstreamConfig};
use netcore::transport::ConnectionOrigin;
use std::{collections::HashMap, time::Duration};

#[test]
fn test_request_manager() {
    let peers = vec![
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
    ];
    let mut request_manager = RequestManager::new(
        UpstreamConfig::default(),
        Duration::from_secs(10),
        Duration::from_secs(30),
        HashMap::new(),
    );
    for peer_id in peers.clone() {
        request_manager.enable_peer(peer_id, ConnectionOrigin::Outbound);
    }

    for _ in 0..50 {
        request_manager.update_score(&peers[0], PeerScoreUpdateType::InvalidChunk);
    }

    let mut pick_counts = HashMap::new();
    for _ in 0..1000 {
        let picked_peer_id = request_manager.pick_peers()[0].clone();
        let counter = pick_counts.entry(picked_peer_id).or_insert(0);
        *counter += 1;
    }

    // unwrap_or needed because the peer with bad score may never be picked, and may be
    // missing from pick_counts
    assert!(pick_counts.get(&peers[0]).unwrap_or(&0) < pick_counts.get(&peers[1]).unwrap());
    assert!(pick_counts.get(&peers[0]).unwrap_or(&0) < pick_counts.get(&peers[2]).unwrap());
    assert!(pick_counts.get(&peers[0]).unwrap_or(&0) < pick_counts.get(&peers[3]).unwrap());
}

#[test]
fn test_remove_requests() {
    let peers = vec![
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
    ];
    let mut request_manager = RequestManager::new(
        UpstreamConfig::default(),
        Duration::from_secs(0),
        Duration::from_secs(30),
        HashMap::new(),
    );
    for peer in peers.iter() {
        request_manager.enable_peer(peer.clone(), ConnectionOrigin::Outbound);
    }

    request_manager.add_request(1, vec![peers[0].clone()]);
    request_manager.add_request(3, vec![peers[1].clone()]);
    request_manager.add_request(5, vec![peers[0].clone()]);
    request_manager.add_request(10, vec![peers[0].clone()]);
    request_manager.add_request(12, vec![peers[1].clone()]);

    request_manager.remove_requests(5);

    assert!(request_manager.get_last_request_time(1).is_none());
    assert!(request_manager.get_last_request_time(3).is_none());
    assert!(request_manager.get_last_request_time(5).is_some());
    assert!(request_manager.get_last_request_time(10).is_some());
    assert!(request_manager.get_last_request_time(12).is_some());
}

#[test]
fn test_request_manager_request_metadata() {
    let peers = vec![
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
    ];
    let mut request_manager = RequestManager::new(
        UpstreamConfig::default(),
        Duration::from_secs(0),
        Duration::from_secs(30),
        HashMap::new(),
    );
    for peer in peers.iter() {
        request_manager.enable_peer(peer.clone(), ConnectionOrigin::Outbound);
    }
    assert!(request_manager.get_first_request_time(1).is_none());

    request_manager.add_request(1, vec![peers[0].clone()]);
    request_manager.check_timeout(1);
    request_manager.add_request(1, vec![peers[1].clone()]);
    assert!(request_manager.peer_score(&peers[0]).unwrap() < 99.0);
    assert!(request_manager.peer_score(&peers[1]).unwrap() > 99.0);
    assert!(
        request_manager.get_first_request_time(1).unwrap()
            <= request_manager.get_last_request_time(1).unwrap()
    );
}
