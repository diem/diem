// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::request_manager::RequestManager;
use diem_config::config::{PeerNetworkId, UpstreamConfig};
use netcore::transport::ConnectionOrigin;
use std::{collections::HashMap, time::Duration};

const NUM_CHUNKS_TO_PROCESS: u64 = 50;
const NUM_PICKS_TO_MAKE: u64 = 1000;

#[test]
fn test_invalid_chunk() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(10, 4);

    // Process multiple invalid chunk responses from validator 0
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        request_manager.process_invalid_chunk(&validators[0]);
    }

    // Calculate pick counts for the validators
    let pick_counts = calculate_pick_counts_for_validators(&mut request_manager, NUM_PICKS_TO_MAKE);

    // Verify validator 0 is chosen less often than the other validators
    let validator_0_count = pick_counts.get(&validators[0]).unwrap_or(&0);
    assert!(validator_0_count < pick_counts.get(&validators[1]).unwrap());
    assert!(validator_0_count < pick_counts.get(&validators[2]).unwrap());
    assert!(validator_0_count < pick_counts.get(&validators[3]).unwrap());
}

#[test]
fn test_remove_requests() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(0, 2);

    let validator_0 = vec![validators[0].clone()];
    let validator_1 = vec![validators[1].clone()];

    // Add version requests to request manager
    request_manager.add_request(1, validator_0.clone());
    request_manager.add_request(3, validator_1.clone());
    request_manager.add_request(5, validator_0.clone());
    request_manager.add_request(10, validator_0);
    request_manager.add_request(12, validator_1);

    // Remove all request versions below 5
    request_manager.remove_requests(5);

    // Verify versions updated correctly
    assert!(request_manager.get_last_request_time(1).is_none());
    assert!(request_manager.get_last_request_time(3).is_none());
    assert!(request_manager.get_last_request_time(5).is_some());
    assert!(request_manager.get_last_request_time(10).is_some());
    assert!(request_manager.get_last_request_time(12).is_some());
}

#[test]
fn test_request_metadata() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(0, 2);

    let validator_0 = validators[0].clone();
    let validator_1 = validators[1].clone();

    // Verify first request time doesn't exist for missing request
    assert!(request_manager.get_first_request_time(1).is_none());

    // Add versions requests to request manager
    request_manager.add_request(1, vec![validator_0.clone()]);
    request_manager.check_timeout(1).unwrap();
    request_manager.add_request(1, vec![validator_1.clone()]);

    // Verify scores are affected by request timeouts and that request metadata is updated
    assert!(request_manager.peer_score(&validator_0).unwrap() < 99.0);
    assert!(request_manager.peer_score(&validator_1).unwrap() > 99.0);
    assert!(
        request_manager.get_first_request_time(1).unwrap()
            <= request_manager.get_last_request_time(1).unwrap()
    );
}

/// Picks a peer to send a chunk request to (multiple times) and constructs a pick count
/// for each of the chosen peers.
fn calculate_pick_counts_for_validators(
    request_manager: &mut RequestManager,
    number_of_picks_to_execute: u64,
) -> HashMap<PeerNetworkId, u64> {
    let mut pick_counts = HashMap::new();

    for _ in 0..number_of_picks_to_execute {
        let picked_peers = request_manager.pick_peers();
        assert_eq!(1, picked_peers.len()); // Ensure only one validator per multicast level

        let picked_peer = picked_peers[0].clone();
        let counter = pick_counts.entry(picked_peer).or_insert(0);
        *counter += 1;
    }

    pick_counts
}

/// Generates a new request manager with a specified number of validator peers enabled.
fn generate_request_manager_and_test_validators(
    request_timeout: u64,
    num_validators: u64,
) -> (RequestManager, Vec<PeerNetworkId>) {
    let mut request_manager = RequestManager::new(
        UpstreamConfig::default(),
        Duration::from_secs(request_timeout),
        Duration::from_secs(30),
        HashMap::new(),
    );

    let mut validators = Vec::new();
    for _ in 0..num_validators {
        let validator = PeerNetworkId::random_validator();
        request_manager.enable_peer(validator.clone(), ConnectionOrigin::Outbound);
        validators.push(validator);
    }

    (request_manager, validators)
}
