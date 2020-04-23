// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_manager::{PeerManager, PeerScoreUpdateType};
use libra_config::config::{PeerNetworkId, UpstreamConfig};
use std::collections::HashMap;

#[test]
fn test_peer_manager() {
    let peers = vec![
        PeerNetworkId::random(),
        PeerNetworkId::random(),
        PeerNetworkId::random(),
        PeerNetworkId::random(),
    ];
    let mut upstream_config = UpstreamConfig::default();
    upstream_config.upstream_peers = peers.iter().cloned().collect();
    let mut peer_manager = PeerManager::new(upstream_config);
    for peer_id in peers.clone() {
        peer_manager.enable_peer(peer_id);
    }

    for _ in 0..50 {
        peer_manager.update_score(&peers[0], PeerScoreUpdateType::InvalidChunk);
    }

    let mut pick_counts = HashMap::new();
    for _ in 0..1000 {
        let picked_peer_id = peer_manager.pick_peer().unwrap();
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
    let peers = vec![PeerNetworkId::random(), PeerNetworkId::random()];
    let mut upstream_config = UpstreamConfig::default();
    upstream_config.upstream_peers = peers.iter().cloned().collect();
    let mut peer_manager = PeerManager::new(upstream_config);

    peer_manager.process_request(1, peers[0]);
    peer_manager.process_request(3, peers[1]);
    peer_manager.process_request(5, peers[0]);
    peer_manager.process_request(10, peers[0]);
    peer_manager.process_request(12, peers[1]);

    peer_manager.remove_requests(5);

    assert!(peer_manager.get_last_request_time(1).is_none());
    assert!(peer_manager.get_last_request_time(3).is_none());
    assert!(peer_manager.get_last_request_time(5).is_none());
    assert!(peer_manager.get_last_request_time(10).is_some());
    assert!(peer_manager.get_last_request_time(12).is_some());
}

#[test]
fn test_peer_manager_request_metadata() {
    let peers = vec![PeerNetworkId::random(), PeerNetworkId::random()];
    let mut upstream_config = UpstreamConfig::default();
    upstream_config.upstream_peers = peers.iter().cloned().collect();
    let mut peer_manager = PeerManager::new(upstream_config);
    assert!(peer_manager.get_first_request_time(1).is_none());
    peer_manager.process_request(1, peers[0]);
    peer_manager.process_timeout(1, true);
    peer_manager.process_request(1, peers[1]);
    assert!(peer_manager.peer_score(&peers[0]).unwrap() < 99.0);
    assert!(peer_manager.peer_score(&peers[1]).unwrap() > 99.0);
    assert!(
        peer_manager.get_first_request_time(1).unwrap()
            <= peer_manager.get_last_request_time(1).unwrap()
    );
}
