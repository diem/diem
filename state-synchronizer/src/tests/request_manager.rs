// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::request_manager::RequestManager;
use diem_config::config::{PeerNetworkId, UpstreamConfig};
use netcore::transport::ConnectionOrigin;
use std::{collections::HashMap, time::Duration};
const NUM_CHUNKS_TO_PROCESS: u64 = 50;
const NUM_PICKS_TO_MAKE: u64 = 1000;

#[test]
fn test_chunk_success() {
    let num_validators = 4;
    let (mut request_manager, validators) =
        generate_request_manager_and_test_validators(0, num_validators);

    // Process empty chunk responses from all validators
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        for validator_index in 0..num_validators {
            request_manager.process_empty_chunk(&validators[validator_index as usize]);
        }
    }

    // Process successful chunk responses from validator 0
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        request_manager.process_success_response(&validators[0]);
    }

    // Calculate selection counts for validators
    let pick_counts = calculate_pick_counts_for_validators(&mut request_manager, NUM_PICKS_TO_MAKE);

    // Verify validator 0 is chosen more often than the other validators
    let validator_0_count = pick_counts.get(&validators[0]).unwrap_or(&0);
    for (validator_index, validator) in validators.iter().enumerate() {
        if validator_index != 0 {
            assert!(validator_0_count > pick_counts.get(&validator).unwrap());
        }
    }
}

#[test]
fn test_chunk_timeout() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(0, 4);

    let validator_0 = vec![validators[0].clone()];

    // Process multiple request timeouts from validator 0
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        request_manager.add_request(1, validator_0.clone());
        request_manager.check_timeout(1);
    }

    // Verify validator 0 is chosen less often than the other validators
    verify_bad_validator_picked_less_often(&mut request_manager, &validators, 0);
}

#[test]
fn test_chunk_version_mismatch() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(0, 4);

    let validator_0 = validators[0].clone();

    // Process multiple chunk version mismatches from validator 0
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        request_manager.add_request(100, vec![validator_0.clone()]);
        request_manager
            .process_chunk_version_mismatch(&validator_0, 100, 0)
            .unwrap_err();
    }

    // Verify validator 0 is chosen less often than the other validators
    verify_bad_validator_picked_less_often(&mut request_manager, &validators, 0);
}

#[test]
fn test_empty_chunk() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(10, 4);

    // Process multiple empty chunk responses from validator 0
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        request_manager.process_empty_chunk(&validators[0]);
    }

    // Verify validator 0 is chosen less often than the other validators
    verify_bad_validator_picked_less_often(&mut request_manager, &validators, 0);
}

#[test]
fn test_invalid_chunk() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(10, 4);

    // Process multiple invalid chunk responses from validator 0
    for _ in 0..NUM_CHUNKS_TO_PROCESS {
        request_manager.process_invalid_chunk(&validators[0]);
    }

    // Verify validator 0 is chosen less often than the other validators
    verify_bad_validator_picked_less_often(&mut request_manager, &validators, 0);
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
fn test_request_times() {
    let (mut request_manager, validators) = generate_request_manager_and_test_validators(0, 2);

    // Verify first request time doesn't exist for missing request
    assert!(request_manager.get_first_request_time(1).is_none());

    // Add version requests to request manager
    request_manager.add_request(1, vec![validators[0].clone()]);
    request_manager.add_request(1, vec![validators[1].clone()]);

    // Verify first request time is less than last request time
    assert!(
        request_manager.get_first_request_time(1).unwrap()
            < request_manager.get_last_request_time(1).unwrap()
    );
}

/// Verify that the bad validator is chosen less often than the other validators (due to
/// having a lower peer score internally).
fn verify_bad_validator_picked_less_often(
    request_manager: &mut RequestManager,
    validators: &[PeerNetworkId],
    bad_validator_index: usize,
) {
    // Calculate selection counts for validators
    let pick_counts = calculate_pick_counts_for_validators(request_manager, NUM_PICKS_TO_MAKE);

    // Verify bad validator is chosen less often than the other validators
    let bad_validator_count = pick_counts
        .get(&validators[bad_validator_index])
        .unwrap_or(&0);
    for (validator_index, validator) in validators.iter().enumerate() {
        if validator_index != bad_validator_index {
            assert!(bad_validator_count < pick_counts.get(&validator).unwrap());
        }
    }
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
