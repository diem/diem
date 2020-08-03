// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_manager::{PeerManager, PeerScoreUpdateType};
use libra_config::{
    config::{PeerNetworkId, UpstreamConfig},
    network_id::{NetworkId, NodeNetworkId},
};
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

#[test]
fn test_peer_manager() {
    let peers = vec![
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
    ];
    let mut peer_manager = PeerManager::new(UpstreamConfig::default());
    for peer_id in peers.clone() {
        peer_manager.enable_peer(peer_id, ConnectionOrigin::Outbound);
    }

    for _ in 0..50 {
        peer_manager.update_score(&peers[0], PeerScoreUpdateType::InvalidChunk);
    }

    let mut pick_counts = HashMap::new();
    for _ in 0..1000 {
        let picked_peer_id = peer_manager.pick_peers()[0].clone();
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
    let mut peer_manager = PeerManager::new(UpstreamConfig::default());
    for peer in peers.iter() {
        peer_manager.enable_peer(peer.clone(), ConnectionOrigin::Outbound);
    }

    peer_manager.process_request(1, vec![peers[0].clone()]);
    peer_manager.process_request(3, vec![peers[1].clone()]);
    peer_manager.process_request(5, vec![peers[0].clone()]);
    peer_manager.process_request(10, vec![peers[0].clone()]);
    peer_manager.process_request(12, vec![peers[1].clone()]);

    peer_manager.remove_requests(5, Duration::from_secs(0));

    assert!(peer_manager.get_last_request_time(1).is_none());
    assert!(peer_manager.get_last_request_time(3).is_none());
    assert!(peer_manager.get_last_request_time(5).is_none());
    assert!(peer_manager.get_last_request_time(10).is_some());
    assert!(peer_manager.get_last_request_time(12).is_some());
}

#[test]
fn test_peer_manager_request_metadata() {
    let peers = vec![
        PeerNetworkId::random_validator(),
        PeerNetworkId::random_validator(),
    ];
    let mut peer_manager = PeerManager::new(UpstreamConfig::default());
    for peer in peers.iter() {
        peer_manager.enable_peer(peer.clone(), ConnectionOrigin::Outbound);
    }
    assert!(peer_manager.get_first_request_time(1).is_none());

    peer_manager.process_request(1, vec![peers[0].clone()]);
    peer_manager.process_timeout(1, false);
    peer_manager.process_request(1, vec![peers[1].clone()]);
    assert!(peer_manager.peer_score(&peers[0]).unwrap() < 99.0);
    assert!(peer_manager.peer_score(&peers[1]).unwrap() > 99.0);
    assert!(
        peer_manager.get_first_request_time(1).unwrap()
            <= peer_manager.get_last_request_time(1).unwrap()
    );
}

#[test]
fn test_multicast() {
    let start_time = SystemTime::now();

    // initialize peer manager with proper upstream config
    let upstream_cfg = UpstreamConfig {
        networks: vec![
            NetworkId::Private("vfn".to_string()),
            NetworkId::Private("secondary".to_string()),
            NetworkId::Public,
        ],
    };
    let mut peer_manager = PeerManager::new(upstream_cfg);

    // initialize with some peers
    let primary_network_id = NodeNetworkId::new(NetworkId::Private("vfn".to_string()), 0);
    let failover_network_0 = NodeNetworkId::new(NetworkId::Private("secondary".to_string()), 0);
    let failover_network_1 = NodeNetworkId::new(NetworkId::Public, 0);

    let peers = [
        PeerNetworkId(primary_network_id.clone(), PeerId::random()),
        PeerNetworkId(primary_network_id.clone(), PeerId::random()),
        PeerNetworkId(failover_network_0.clone(), PeerId::random()),
        PeerNetworkId(failover_network_0.clone(), PeerId::random()),
        PeerNetworkId(failover_network_1.clone(), PeerId::random()),
        PeerNetworkId(failover_network_1.clone(), PeerId::random()),
    ];
    for peer in peers.iter() {
        peer_manager.enable_peer(peer.clone(), ConnectionOrigin::Outbound);
    }

    let picked_peer = peer_manager.pick_peers();
    assert_eq!(picked_peer.len(), 1);
    assert_eq!(picked_peer[0].network_id(), primary_network_id);

    // process sent request
    peer_manager.process_request(1, picked_peer.clone());

    // check if is multicast response (should be false, request for version 1 was only sent to one peer)
    assert!(!peer_manager.is_multicast_response(1, &picked_peer[0]));

    // process_timeout, without multicast timeout
    peer_manager.process_timeout(1, false);

    // pick peers
    // shouldn't pick peer in next failover network
    let picked_peer = peer_manager.pick_peers();
    assert_eq!(picked_peer.len(), 1);
    assert_eq!(picked_peer[0].network_id(), primary_network_id);

    // process timeout, with multicast timeout
    peer_manager.process_timeout(1, true);

    let picked_peers = peer_manager.pick_peers();
    assert_eq!(picked_peers.len(), 2);
    assert_eq!(picked_peers[0].network_id(), primary_network_id);
    assert_eq!(picked_peers[1].network_id(), failover_network_0);

    // send and process multicast requests
    peer_manager.remove_requests(1, Duration::from_secs(0));
    peer_manager.process_request(5, picked_peers.clone());

    for peer in picked_peers.iter() {
        assert!(peer_manager.is_multicast_response(5, &peer));
    }

    // process successful chunk from failover peer
    peer_manager.process_success_response(&picked_peers[1]);
    peer_manager.remove_requests(5, start_time.elapsed().unwrap());

    // if primary peer sends chunk after another peer but isn't considered timeout, it should be recognized as a multicast response
    assert!(peer_manager.is_multicast_response(5, &picked_peers[0]));

    // pick peers
    // should give multicast peers
    let picked_peers = peer_manager.pick_peers();
    assert_eq!(picked_peers.len(), 2);
    assert_eq!(picked_peers[0].network_id(), primary_network_id);
    assert_eq!(picked_peers[1].network_id(), failover_network_0);

    peer_manager.process_request(8, picked_peers);

    // this time, time out with multicast timeout
    peer_manager.process_timeout(8, true);

    // picking peer should give us three peers
    let picked_peers = peer_manager.pick_peers();
    assert_eq!(picked_peers.len(), 3);
    let networks = picked_peers
        .iter()
        .map(|peer| peer.network_id())
        .collect::<HashSet<_>>();
    assert!(networks.contains(&primary_network_id));
    assert!(networks.contains(&failover_network_0));
    assert!(networks.contains(&failover_network_1));

    // time out with multicast
    peer_manager.process_request(8, picked_peers);

    // this time, time out with multicast timeout
    peer_manager.process_timeout(8, true);

    // pick peers
    // should only still pick 3, one per network
    let picked_peers = peer_manager.pick_peers();
    assert_eq!(picked_peers.len(), 3);
    let networks = picked_peers
        .iter()
        .map(|peer| peer.network_id())
        .collect::<HashSet<_>>();
    assert!(networks.contains(&primary_network_id));
    assert!(networks.contains(&failover_network_0));
    assert!(networks.contains(&failover_network_1));

    // process successful response from primary upstream
    let primary_peer = picked_peers
        .iter()
        .find(|peer| peer.network_id() == primary_network_id)
        .unwrap();
    peer_manager.process_success_response(primary_peer);
    peer_manager.remove_requests(8, start_time.elapsed().unwrap());

    for peer in picked_peer.iter() {
        if peer.network_id() != primary_network_id {
            assert!(peer_manager.is_multicast_response(8, peer));
        }
    }

    // picking peer after successful multicast response from primary peer
    // we should stop multicasting afterwards
    let picked_peers = peer_manager.pick_peers();
    assert_eq!(picked_peers.len(), 1);
    assert_eq!(picked_peers[0].network_id(), primary_network_id);
}
