// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_harness::default_handler;
use diem_config::{config::RoleType, network_id::NetworkId};
use diem_types::{
    transaction::default_protocol::TransactionListWithProof, waypoint::Waypoint, PeerId,
};
use netcore::transport::ConnectionOrigin::*;
use network::protocols::direct_send::Message;
use state_sync_v1::{error::Error, network::StateSyncMessage};
use std::sync::atomic::{AtomicUsize, Ordering};
use test_harness::StateSyncEnvironment;

mod test_harness;

#[test]
fn test_basic_catch_up() {
    let mut env = StateSyncEnvironment::new(2);

    env.start_validator_peer(0, false);
    env.start_validator_peer(1, false);

    let validator_0 = env.get_state_sync_peer(0);
    let validator_1 = env.get_state_sync_peer(1);

    // Test small sequential syncs, batch sync for multiple transactions and
    // batch sync for multiple chunks.
    let synced_versions = vec![1, 2, 3, 4, 5, 20, 2000];
    for version in synced_versions {
        validator_0.commit(version);
        let target_li = validator_0.latest_li();

        validator_1.sync_to(target_li);
        assert_eq!(validator_1.latest_li().ledger_info().version(), version);
    }
}

#[test]
fn test_basic_catch_up_read_only() {
    let mut env = StateSyncEnvironment::new(2);

    env.setup_state_sync_peer(
        0,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        10_000,
        10_000,
        false,
        true,
    );
    env.start_validator_peer(1, false);

    let validator_1 = env.get_state_sync_peer(0);
    let validator_2 = env.get_state_sync_peer(1);

    // Test small sequential syncs, batch sync for multiple transactions and
    // batch sync for multiple chunks.
    let synced_versions = vec![1, 2, 3, 4, 5, 20, 2000];
    for version in synced_versions {
        validator_1.commit(version);
        let target_li = validator_1.latest_li();

        validator_2.sync_to(target_li);
        assert_eq!(validator_2.latest_li().ledger_info().version(), version);
    }
}

#[test]
fn test_flaky_peer_sync() {
    let mut env = StateSyncEnvironment::new(2);

    env.start_validator_peer(0, false);

    // Create handler that causes error, but has successful retries
    let attempt = AtomicUsize::new(0);
    let handler = Box::new(move |resp| -> Result<TransactionListWithProof, Error> {
        let fail_request = attempt.load(Ordering::Relaxed) == 0;
        attempt.fetch_add(1, Ordering::Relaxed);
        if fail_request {
            Err(Error::UnexpectedError("Failed to fetch chunk!".into()))
        } else {
            Ok(resp)
        }
    });
    env.start_state_sync_peer(1, handler, RoleType::Validator, Waypoint::default(), false);

    let validator_0 = env.get_state_sync_peer(0);
    let validator_1 = env.get_state_sync_peer(1);

    let synced_version = 20;
    validator_0.commit(synced_version);
    validator_1.sync_to(validator_0.latest_li());
    assert_eq!(
        validator_1.latest_li().ledger_info().version(),
        synced_version
    );
}

#[test]
#[should_panic]
fn test_request_timeout() {
    let mut env = StateSyncEnvironment::new(2);

    let handler = Box::new(move |_| -> Result<TransactionListWithProof, Error> {
        Err(Error::UnexpectedError("Failed to fetch chunk!".into()))
    });
    env.start_state_sync_peer(0, handler, RoleType::Validator, Waypoint::default(), false);
    env.setup_state_sync_peer(
        1,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        100,
        300,
        false,
        false,
    );

    let validator_0 = env.get_state_sync_peer(0);
    let validator_1 = env.get_state_sync_peer(1);

    validator_0.commit(1);
    validator_1.sync_to(validator_0.latest_li());
}

#[test]
fn test_read_only_validator() {
    let mut env = StateSyncEnvironment::new(2);

    env.setup_state_sync_peer(
        0,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        10_000,
        1_000_000,
        false,
        true,
    );
    env.start_fullnode_peer(1, false);

    let validator = env.get_state_sync_peer(0);
    let fullnode = env.get_state_sync_peer(1);

    validator.commit(10);
    // first sync should be fulfilled immediately after peer discovery
    assert!(fullnode.wait_for_version(10, None));

    validator.commit(20);
    // second sync will be done via long polling cause first node should send new request
    // after receiving first chunk immediately
    assert!(fullnode.wait_for_version(20, None));
}

#[test]
fn test_full_node() {
    let mut env = StateSyncEnvironment::new(2);

    env.start_validator_peer(0, false);
    env.start_fullnode_peer(1, false);

    let validator = env.get_state_sync_peer(0);
    let fullnode = env.get_state_sync_peer(1);

    validator.commit(10);
    // first sync should be fulfilled immediately after peer discovery
    assert!(fullnode.wait_for_version(10, None));

    validator.commit(20);
    // second sync will be done via long polling cause first node should send new request
    // after receiving first chunk immediately
    assert!(fullnode.wait_for_version(20, None));
}

#[test]
fn catch_up_through_epochs_validators() {
    let mut env = StateSyncEnvironment::new(2);

    env.setup_state_sync_peer(
        0,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        10_000,
        1_000_000,
        false,
        true,
    );
    env.start_validator_peer(1, false);

    let validator_0 = env.get_state_sync_peer(0);
    let validator_1 = env.get_state_sync_peer(1);

    // catch up to the next epoch starting from the middle of the current one
    validator_0.commit(20);
    validator_1.sync_to(validator_0.latest_li());
    validator_0.commit(40);

    let validator_infos = vec![
        validator_0.get_validator_info(),
        validator_1.get_validator_info(),
    ];
    validator_0.move_to_next_epoch(validator_infos, 0);

    validator_0.commit(100);
    validator_1.sync_to(validator_0.latest_li());
    assert_eq!(validator_1.latest_li().ledger_info().version(), 100);
    assert_eq!(validator_1.latest_li().ledger_info().epoch(), 2);

    // catch up through multiple epochs
    for epoch in 2..10 {
        validator_0.commit(epoch * 100);

        let validator_infos = vec![
            validator_0.get_validator_info(),
            validator_1.get_validator_info(),
        ];
        validator_0.move_to_next_epoch(validator_infos, 0);
    }
    validator_0.commit(950); // At this point peer 0 is at epoch 10 and version 950
    validator_1.sync_to(validator_0.latest_li());
    assert_eq!(validator_1.latest_li().ledger_info().version(), 950);
    assert_eq!(validator_1.latest_li().ledger_info().epoch(), 10);
}

#[test]
fn catch_up_through_epochs_full_node() {
    let mut env = StateSyncEnvironment::new(3);

    env.start_validator_peer(0, false);
    let validator_0 = env.get_state_sync_peer(0);

    // catch up through multiple epochs
    for epoch in 1..10 {
        validator_0.commit(epoch * 100);
        validator_0.move_to_next_epoch(vec![validator_0.get_validator_info().clone()], 0);
    }
    validator_0.commit(950); // At this point validator_0 is at epoch 10 and version 950
    drop(validator_0);

    env.start_fullnode_peer(1, false);
    let fullnode = env.get_state_sync_peer(1);

    assert!(fullnode.wait_for_version(950, None));
    assert_eq!(fullnode.latest_li().ledger_info().epoch(), 10);
    drop(fullnode);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_fullnode_peer(2, false);
    let fullnode = env.get_state_sync_peer(2);

    assert!(fullnode.wait_for_version(950, None));
    assert_eq!(fullnode.latest_li().ledger_info().epoch(), 10);
}

#[test]
fn catch_up_with_waypoints() {
    let mut env = StateSyncEnvironment::new(3);

    env.start_validator_peer(0, false);
    let validator_0 = env.get_state_sync_peer(0);

    let mut curr_version = 0;
    for _ in 1..10 {
        curr_version += 100;
        validator_0.commit(curr_version);

        validator_0.move_to_next_epoch(vec![validator_0.get_validator_info().clone()], 0);

        curr_version += 400;
        // this creates an epoch that spans >1 chunk (chunk_size = 250)
        validator_0.commit(curr_version);

        validator_0.move_to_next_epoch(vec![validator_0.get_validator_info().clone()], 0);
    }
    validator_0.commit(5250); // At this point validator is at epoch 19 and version 5250

    // Create a waypoint based on LedgerInfo of peer 0 at version 3500 (epoch 14)
    let waypoint_li = validator_0.get_epoch_ending_ledger_info(3500);
    let waypoint = Waypoint::new_epoch_boundary(waypoint_li.ledger_info()).unwrap();
    drop(validator_0);

    env.start_state_sync_peer(1, default_handler(), RoleType::FullNode, waypoint, false);
    let fullnode = env.get_state_sync_peer(1);
    fullnode.wait_until_initialized();
    assert!(fullnode.latest_li().ledger_info().version() >= 3500);
    assert!(fullnode.latest_li().ledger_info().epoch() >= 14);

    // Once caught up with the waypoint fullnode 1 continues with the regular state sync
    assert!(fullnode.wait_for_version(5250, None));
    assert_eq!(fullnode.latest_li().ledger_info().epoch(), 19);
    drop(fullnode);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_fullnode_peer(2, false);
    let fullnode = env.get_state_sync_peer(2);
    assert!(fullnode.wait_for_version(5250, None));
    assert_eq!(fullnode.latest_li().ledger_info().epoch(), 19);
}

#[test]
fn test_lagging_fullnode_read_only() {
    let mut env = StateSyncEnvironment::new(3);

    // Start 1 validator and 2 fullnodes (fullnode_1 is read-only)
    env.start_validator_peer(0, true);
    env.start_fullnode_peer(1, true);
    env.setup_state_sync_peer(
        2,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        10_000,
        1_000_000,
        true,
        true,
    );

    let validator_0 = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(1);
    let fullnode_1 = env.get_state_sync_peer(2);

    // Get peer ids of nodes (across different networks)
    let validator_0_peer_id = validator_0.get_peer_id(NetworkId::Validator);
    let fullnode_0_peer_id_vfn = fullnode_0.get_peer_id(NetworkId::Vfn);
    let fullnode_0_peer_id_pfn = fullnode_0.get_peer_id(NetworkId::Public);
    let fullnode_1_peer_id_pfn = fullnode_1.get_peer_id(NetworkId::Public);

    // Commit version 400 at the validator
    validator_0.commit(400);
    drop(validator_0);
    drop(fullnode_0);
    drop(fullnode_1);

    // Validator and fullnode_0 discover each other
    send_connection_notifications(&mut env, validator_0_peer_id, fullnode_0_peer_id_vfn, true);

    // Fullnodes discover each other
    send_connection_notifications(
        &mut env,
        fullnode_1_peer_id_pfn,
        fullnode_0_peer_id_pfn,
        true,
    );

    // Deliver fullnode_0 messages and verify versions and targets
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_vfn);
    check_chunk_request(message, 0, None);
    let (_, message) = env.deliver_msg(validator_0_peer_id);
    check_chunk_response(message, 400, 1, 250);
    env.get_state_sync_peer(2).wait_for_version(250, None);

    // Validator loses fullnode_0
    send_connection_notifications(&mut env, validator_0_peer_id, fullnode_0_peer_id_vfn, false);

    // Fullnode_0 sends chunk request to fullnode_1
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 250, Some(400));

    // Validator 0 commits to new version and fullnode 1 is fast forwarded
    env.get_state_sync_peer(0).commit(500);
    env.clone_storage(0, 2);
    env.get_state_sync_peer(2).wait_for_version(500, Some(500));

    // Fullnode 0 sends chunk request to fullnode 1 and commits to latest state
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 250, Some(400));
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 400, 251, 150);

    // Validator 0 commits to new version and fullnode 1 is fast forwarded
    env.get_state_sync_peer(0).commit(1000);
    env.clone_storage(0, 2);
    env.get_state_sync_peer(2)
        .wait_for_version(1000, Some(1000));

    // Fullnode 0 sends chunk requests to fullnode 1 and commits to latest state
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 400, None);
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 1000, 401, 250);

    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 650, None);
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 1000, 651, 250);

    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 900, Some(1000));
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 1000, 901, 100);
}

#[test]
fn test_lagging_upstream_long_poll() {
    let mut env = StateSyncEnvironment::new(4);

    // Start 2 validators and 2 fullnodes
    env.setup_state_sync_peer(
        0,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        10_000,
        10_000,
        true,
        true,
    );
    env.start_validator_peer(1, true);
    env.setup_state_sync_peer(
        2,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        10_000,
        1_000_000,
        true,
        false,
    );
    env.start_state_sync_peer(
        3,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        true,
    );

    let validator_0 = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(2);
    let fullnode_1 = env.get_state_sync_peer(3);

    // Get peer ids of nodes (across different networks)
    let validator_0_peer_id = validator_0.get_peer_id(NetworkId::Validator);
    let fullnode_0_peer_id_vfn = fullnode_0.get_peer_id(NetworkId::Vfn);
    let fullnode_0_peer_id_pfn = fullnode_0.get_peer_id(NetworkId::Public);
    let fullnode_1_peer_id_vfn = fullnode_1.get_peer_id(NetworkId::Vfn);
    let fullnode_1_peer_id_pfn = fullnode_1.get_peer_id(NetworkId::Public);

    // Commit version 400 at the validator
    validator_0.commit(400);
    drop(validator_0);
    drop(fullnode_0);
    drop(fullnode_1);

    // Validator and fullnode discover each other
    send_connection_notifications(&mut env, validator_0_peer_id, fullnode_0_peer_id_vfn, true);

    // Fullnodes discover each other
    send_connection_notifications(
        &mut env,
        fullnode_1_peer_id_pfn,
        fullnode_0_peer_id_pfn,
        true,
    );

    // Deliver messages and verify versions and targets
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_vfn);
    check_chunk_request(message, 0, None);
    let (_, message) = env.deliver_msg(validator_0_peer_id);
    check_chunk_response(message, 400, 1, 250);
    env.get_state_sync_peer(2).wait_for_version(250, None);

    // Validator loses fullnode
    send_connection_notifications(&mut env, validator_0_peer_id, fullnode_0_peer_id_vfn, false);

    // Fullnode sends chunk request to other fullnode
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 250, Some(400));

    // Validator 0 commits to new version and fullnode 1 is fast forwarded
    env.get_state_sync_peer(0).commit(500);
    env.clone_storage(0, 3);
    env.get_state_sync_peer(3).wait_for_version(500, Some(500));

    // Connect the validator and the failover fullnode so the fullnode can sync.
    send_connection_notifications(&mut env, validator_0_peer_id, fullnode_1_peer_id_vfn, true);

    // Trigger another commit so that the fullnodes's commit will trigger subscription delivery
    env.get_state_sync_peer(0).commit(600);
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_vfn);
    check_chunk_request(message, 500, None);
    let (_, message) = env.deliver_msg(validator_0_peer_id);
    check_chunk_response(message, 600, 501, 100);

    // Fullnode 1 sends long-poll subscription to fullnode 0
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 400, 251, 150);

    // Fullnode 0 sends chunk request to fullnode 1 and commits to latest state
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 400, None);
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 600, 401, 200);
    env.get_state_sync_peer(2).wait_for_version(600, Some(600));
}

#[test]
fn test_fullnode_catch_up_moving_target_epochs() {
    // Create validator and fullnode
    let mut env = StateSyncEnvironment::new(2);
    env.start_validator_peer(0, true);
    env.start_fullnode_peer(1, true);

    // Get peer ids of nodes
    let validator_peer_id = env.get_state_sync_peer(0).get_peer_id(NetworkId::Validator);
    let fullnode_peer_id = env.get_state_sync_peer(1).get_peer_id(NetworkId::Vfn);

    // Validator and fullnode discover each other.
    send_connection_notifications(&mut env, validator_peer_id, fullnode_peer_id, true);

    // Versions to be committed by the validator
    let commit_versions = vec![
        900, 1800, 2800, 3100, 3200, 3300, 3325, 3350, 3400, 3450, 3650, 4300, 4549,
    ];

    // Versions that will move to the next epoch
    let versions_for_new_epochs = vec![2800, 3325, 4300];

    // Expected fullnode sync states (i.e., synced version and committed version)
    let expected_states = vec![
        (250, 0),
        (500, 0),
        (750, 0),
        (900, 900),
        (1150, 900),
        (1400, 900),
        (1650, 900),
        (1900, 900),
        (2150, 900),
        (2400, 900),
        (2650, 900),
        (2800, 2800),
        (3050, 2800),
        (3300, 2800),
        (3325, 3325),
        (3575, 3325),
        (3825, 3325),
        (4075, 3325),
        (4300, 4300),
        (4549, 4549),
    ];

    // Update the versions at the validator and check the full node syncs correctly.
    for (index, (synced_version, committed_version)) in expected_states.iter().enumerate() {
        if let Some(committed_version) = commit_versions.get(index) {
            let validator = env.get_state_sync_peer(0);
            validator.commit(*committed_version);

            if versions_for_new_epochs.contains(committed_version) {
                let validator_infos = vec![validator.get_validator_info()];
                validator.move_to_next_epoch(validator_infos, 0);
            }
        }

        env.deliver_msg(fullnode_peer_id);
        env.deliver_msg(validator_peer_id);

        let fullnode = env.get_state_sync_peer(1);
        if !fullnode.wait_for_version(*synced_version, Some(*committed_version)) {
            panic!(
                "Failed to reach synced version: {} and committed version: {}",
                synced_version, committed_version
            );
        }
    }
}

#[test]
fn test_fullnode_catch_up_moving_target() {
    // Create validator and fullnode
    let mut env = StateSyncEnvironment::new(2);
    env.start_validator_peer(0, true);
    env.start_fullnode_peer(1, true);

    // Get peer ids of nodes
    let validator_peer_id = env.get_state_sync_peer(0).get_peer_id(NetworkId::Validator);
    let fullnode_peer_id = env.get_state_sync_peer(1).get_peer_id(NetworkId::Vfn);

    // Validator and fullnode discover each other.
    send_connection_notifications(&mut env, validator_peer_id, fullnode_peer_id, true);

    // Expected fullnode sync states (i.e., synced version and committed version)
    let expected_states = vec![
        (250, 0),
        (500, 0),
        (750, 0),
        (1000, 1000),
        (1250, 1000),
        (1500, 1000),
        (1750, 1000),
        (2000, 2000),
        (2250, 2000),
        (2500, 2000),
        (2750, 2000),
        (3000, 2000),
        (3250, 2000),
        (3500, 2000),
        (3750, 2000),
        (4000, 4000),
    ];

    // Update the versions at the validator and check the full node syncs correctly.
    // Every iteration the validator commits another 500 transactions (i.e., the validator
    // is committing at twice the rate the full node can sync).
    for (index, (synced_version, committed_version)) in expected_states.iter().enumerate() {
        let next_commit = (index + 1) * 500;
        env.get_state_sync_peer(0).commit(next_commit as u64);

        env.deliver_msg(fullnode_peer_id);
        env.deliver_msg(validator_peer_id);

        let fullnode = env.get_state_sync_peer(1);
        if !fullnode.wait_for_version(*synced_version, Some(*committed_version)) {
            panic!(
                "Failed to reach synced version: {} and committed version: {}",
                synced_version, committed_version
            );
        }
    }
}

// Unfortunately, this test contains race conditions that are inherent to the way
// the integration tests are built. While flakes don't seem to occur locally
// (or often), we've hit them at least once in CI, so let's ignore this for now and
// return to fix it once this is higher priority.
// Note: this test is still useful for verifying state sync behaviour.
#[test]
#[ignore]
fn test_fn_failover() {
    let mut env = StateSyncEnvironment::new(5);

    // Start a validator
    env.start_validator_peer(0, true);

    // Start a fullnode with two upstream networks
    env.setup_state_sync_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        60_000,
        true,
        false,
    );

    // Start up 3 PFNs
    env.start_fullnode_peer(2, true);
    env.start_fullnode_peer(3, true);
    env.start_fullnode_peer(4, true);

    let validator = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(1);
    let fullnode_1 = env.get_state_sync_peer(2);
    let fullnode_2 = env.get_state_sync_peer(3);
    let fullnode_3 = env.get_state_sync_peer(4);

    // Grab the nodes peer id's
    let validator_peer_id = validator.get_peer_id(NetworkId::Vfn);
    let fn_0_vfn_peer_id = fullnode_0.get_peer_id(NetworkId::Vfn);
    let fn_0_pfn_peer_id = fullnode_0.get_peer_id(NetworkId::Public);
    let fn_1_pfn_peer_id = fullnode_1.get_peer_id(NetworkId::Public);
    let fn_2_pfn_peer_id = fullnode_1.get_peer_id(NetworkId::Public);
    let fn_3_pfn_peer_id = fullnode_3.get_peer_id(NetworkId::Public);

    drop(validator);
    drop(fullnode_0);
    drop(fullnode_1);
    drop(fullnode_2);
    drop(fullnode_3);

    // Validator discovers fullnode 0
    send_connection_notifications(&mut env, validator_peer_id, fn_0_vfn_peer_id, true);

    // Set up the PFN network: fullnode 0 sends a new peer event to all upstream PFNs
    let upstream_peer_ids = [fn_1_pfn_peer_id, fn_2_pfn_peer_id, fn_3_pfn_peer_id];
    for peer in upstream_peer_ids.iter() {
        send_connection_notifications(&mut env, *peer, fn_0_pfn_peer_id, true);
    }

    // Commit transactions on the validator and verify that fullnode 0 sends chunk requests
    // to the validator only.
    let responding_peer_id = execute_commit_and_verify_chunk_requests(
        &mut env,
        1,
        &[fn_0_vfn_peer_id],
        &[validator_peer_id],
        &[fn_0_pfn_peer_id],
        false,
    );

    // Bring down the validator connection to fullnode 0 and deliver the last chunk response
    send_connection_notifications(&mut env, validator_peer_id, fn_0_vfn_peer_id, false);
    env.deliver_msg(responding_peer_id);

    // Check that fullnode 0 sends chunk requests to the PFNs only
    let responding_peer_id = execute_commit_and_verify_chunk_requests(
        &mut env,
        2,
        &[fn_0_pfn_peer_id],
        &upstream_peer_ids,
        &[fn_0_vfn_peer_id],
        false,
    );

    // Disconnect fullnode 0 from fullnode 1 and fullnode 2
    send_connection_notifications(&mut env, fn_1_pfn_peer_id, fn_0_pfn_peer_id, false);
    send_connection_notifications(&mut env, fn_2_pfn_peer_id, fn_0_pfn_peer_id, false);

    // Deliver the last chunk response to fullnode 0
    let (chunk_response_recipient, _) = env.deliver_msg(responding_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_pfn_peer_id);

    // Verify fullnode 0 only broadcasts to the single live fallback peer (fullnode 3)
    let responding_peer_id = execute_commit_and_verify_chunk_requests(
        &mut env,
        3,
        &[fn_0_pfn_peer_id],
        &[fn_3_pfn_peer_id],
        &[fn_0_vfn_peer_id],
        false,
    );

    // Disconnect fullnode 0 and fullnode 3 and deliver the last chunk response
    send_connection_notifications(&mut env, fn_3_pfn_peer_id, fn_0_pfn_peer_id, false);
    let (chunk_response_recipient, _) = env.deliver_msg(responding_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_pfn_peer_id);

    // Verify that no sync requests are sent (as all upstream peers are down)
    env.assert_no_message_sent(fn_0_vfn_peer_id);
    env.assert_no_message_sent(fn_0_pfn_peer_id);

    // Connect fullnode 0 and fullnode 2
    send_connection_notifications(&mut env, fn_2_pfn_peer_id, fn_0_pfn_peer_id, true);

    // Verify that we only broadcast to the single live fallback peer (fullnode 2)
    let responding_peer_id = execute_commit_and_verify_chunk_requests(
        &mut env,
        4,
        &[fn_0_pfn_peer_id],
        &[fn_2_pfn_peer_id],
        &[fn_0_vfn_peer_id],
        false,
    );

    // Connect fullnode 0 and validator and deliver the last chunk response
    send_connection_notifications(&mut env, validator_peer_id, fn_0_vfn_peer_id, true);
    let (chunk_response_recipient, _) = env.deliver_msg(responding_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_pfn_peer_id);

    // Check that fullnode 0 sends chunk requests to the validator and fullnode 2
    let responding_peer_id = execute_commit_and_verify_chunk_requests(
        &mut env,
        5,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        false,
    );

    // Bring back all PFNs and deliver the last chunk response
    for peer in &[fn_1_pfn_peer_id, fn_3_pfn_peer_id] {
        send_connection_notifications(&mut env, *peer, fn_0_pfn_peer_id, true);
    }
    env.deliver_msg(responding_peer_id);

    // Check that fullnode 0 sends chunk requests to the validator and fullnode 2
    // (because of optimistic fetch)
    let responding_peer_id = execute_commit_and_verify_chunk_requests(
        &mut env,
        6,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        false,
    );

    // Deliver the last chunk response and verify that fullnode 0 is the target
    let (chunk_response_recipient, _) = env.deliver_msg(responding_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_pfn_peer_id);

    // Check that fullnode 0 sends chunk requests to the validator only
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        7,
        &[fn_0_vfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[fn_0_pfn_peer_id],
        false,
    );
}

// This test helper is used to execute a commit (at commit_version) on a validator node. It
// verifies that requesting_peers send chunk requests to one of the specified
// target_peers, and that the inactive_peers don't send any chunk requests.
// Note: if skip_all_responses is true, this helper doesn't deliver a single chunk response to
// the requesting_peer. Otherwise, all responses are sent, except the chunk from the last
// responding node. In all cases, we return the peer_id of the last node that should have
// responded to the requesting_peer so that it can be used to deliver the last chunk manually.
fn execute_commit_and_verify_chunk_requests(
    env: &mut StateSyncEnvironment,
    commit_version: u64,
    requesting_peers: &[PeerId],
    target_peers: &[PeerId],
    inactive_peers: &[PeerId],
    skip_all_responses: bool,
) -> PeerId {
    let validator_index = 0;
    let pfn_indices = [2, 3, 4];

    // Execute a new commit at the next version
    env.get_state_sync_peer(validator_index)
        .commit(commit_version);

    // Automatically sync up all PFNs to the validator's state
    for pfn in &pfn_indices {
        env.clone_storage(validator_index, *pfn);
    }

    // Deliver the chunk requests and verify the target recipients
    let mut request_recipients = vec![];
    for requesting_peer in requesting_peers {
        let (recipient, _) = env.deliver_msg(*requesting_peer);
        assert!(target_peers.contains(&recipient));
        request_recipients.push(recipient);
    }

    // Verify no requests are sent by the inactive_peers
    for inactive_peer in inactive_peers {
        env.assert_no_message_sent(*inactive_peer);
    }

    if skip_all_responses {
        // Return the last recipient
        return *request_recipients.last().unwrap();
    } else {
        // Deliver the chunk responses (except for the last chunk)
        let number_of_recipients = request_recipients.len();
        for (recipient_id, recipient) in request_recipients.iter().enumerate() {
            if recipient_id < number_of_recipients - 1 {
                env.deliver_msg(*recipient);
            } else {
                // Don't deliver the last chunk
                return *recipient;
            }
        }
    }

    panic!("Unexpected code path in test helper! No chunk responses were sent?");
}

// Unfortunately, this test contains race conditions that are inherent to the way
// the integration tests are built. While flakes don't seem to occur locally
// (or often), we've hit them at least once in CI, so let's ignore this for now and
// return to fix it once this is higher priority.
// Note: this test is still useful for verifying state sync behaviour.
#[test]
#[ignore]
fn test_multicast_failover() {
    let mut env = StateSyncEnvironment::new(5);

    // Start a validator
    env.start_validator_peer(0, true);

    // Start a fullnode with two upstream networks
    let multicast_timeout_ms = 3_000;
    env.setup_state_sync_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        multicast_timeout_ms,
        true,
        false,
    );

    // Start up 3 FNs
    env.start_fullnode_peer(2, true);
    env.start_fullnode_peer(3, true);
    env.start_fullnode_peer(4, true);

    let validator = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(1);
    let fullnode_1 = env.get_state_sync_peer(2);
    let fullnode_2 = env.get_state_sync_peer(3);

    // Grab the nodes peer id's
    let validator_peer_id = validator.get_peer_id(NetworkId::Vfn);
    let fn_0_vfn_peer_id = fullnode_0.get_peer_id(NetworkId::Vfn);
    let fn_0_pfn_peer_id = fullnode_0.get_peer_id(NetworkId::Public);
    let fn_2_pfn_peer_id = fullnode_2.get_peer_id(NetworkId::Public);

    drop(validator);
    drop(fullnode_0);
    drop(fullnode_1);
    drop(fullnode_2);

    // Fullnode 0 discovers validator, fullnode 1 and fullnode 2
    send_connection_notifications(&mut env, validator_peer_id, fn_0_vfn_peer_id, true);
    send_connection_notifications(&mut env, fn_2_pfn_peer_id, fn_0_pfn_peer_id, true);

    // Verify that fullnode 0 only broadcasts to the single validator peer
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        1,
        &[fn_0_vfn_peer_id],
        &[validator_peer_id],
        &[fn_0_pfn_peer_id],
        true,
    );

    // Wait for fullnode 0's chunk request to time out before delivering the response
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));
    env.deliver_msg(validator_peer_id);

    // Verify that fullnode 0 broadcasts to the validator peer and both fallback peers
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        2,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        true,
    );

    // Wait for fullnode 0's chunk request to time out before delivering responses
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_2_pfn_peer_id);

    // Verify that fullnode 0 broadcasts to the validator peer and both fallback peers
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        3,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        true,
    );

    // Deliver chunks from the 3 nodes (PFN delivers before validator)
    env.deliver_msg(fn_2_pfn_peer_id);
    env.deliver_msg(validator_peer_id);

    // Verify that fullnode 0 still broadcasts to the validator peer and both fallback peers
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        4,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        true,
    );

    // Deliver chunks from the 3 nodes (PFN delivers before validator)
    env.deliver_msg(fn_2_pfn_peer_id);
    env.deliver_msg(validator_peer_id);

    // Verify that fullnode 0 still broadcasts to the validator peer and both fallback peers
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        5,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        true,
    );

    // Deliver chunks from all nodes (with the validator as the first responder)
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_2_pfn_peer_id);

    // Verify that fullnode 0 still broadcasts to the validator peer and both fallback peers
    // as optimistic fetch hasn't processed the response from the validator
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        6,
        &[fn_0_vfn_peer_id, fn_0_pfn_peer_id],
        &[validator_peer_id, fn_2_pfn_peer_id],
        &[],
        true,
    );

    // Deliver chunks from all nodes (with the validator as the first responder)
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_2_pfn_peer_id);

    // Verify that fullnode 0 now only broadcasts to the validator
    let _ = execute_commit_and_verify_chunk_requests(
        &mut env,
        7,
        &[fn_0_vfn_peer_id],
        &[validator_peer_id],
        &[fn_0_pfn_peer_id],
        true,
    );
}

fn check_chunk_request(message: Message, known_version: u64, target_version: Option<u64>) {
    let chunk_request: StateSyncMessage = bcs::from_bytes(&message.mdata).unwrap();
    match chunk_request {
        StateSyncMessage::GetChunkRequest(chunk_request) => {
            assert_eq!(chunk_request.known_version, known_version);
            assert_eq!(chunk_request.target.version(), target_version);
        }
        StateSyncMessage::GetChunkResponse(_) => {
            panic!("Received chunk response but expecting chunk request!");
        }
    }
}

fn check_chunk_response(
    message: Message,
    response_li_version: u64,
    chunk_start_version: u64,
    chunk_length: usize,
) {
    let chunk_response: StateSyncMessage = bcs::from_bytes(&message.mdata).unwrap();
    match chunk_response {
        StateSyncMessage::GetChunkRequest(_) => {
            panic!("Received chunk response but expecting chunk request!");
        }
        StateSyncMessage::GetChunkResponse(chunk_response) => {
            assert_eq!(chunk_response.response_li.version(), response_li_version);
            assert_eq!(
                chunk_response
                    .txn_list_with_proof
                    .first_transaction_version
                    .unwrap(),
                chunk_start_version
            );
            assert_eq!(
                chunk_response.txn_list_with_proof.transactions.len(),
                chunk_length
            )
        }
    }
}

// Sends a connection notification to the given peers (i.e., connecting/disconnecting peer_id_0 and
// peer_id_1). If `new_peer_notification` is true, the connection notification is a "new peer"
// (connect) notification, otherwise, a "lost peer" (disconnect) notification is sent.
fn send_connection_notifications(
    env: &mut StateSyncEnvironment,
    peer_id_0: PeerId,
    peer_id_1: PeerId,
    new_peer_notification: bool,
) {
    env.send_peer_event(peer_id_0, peer_id_1, new_peer_notification, Outbound);
    env.send_peer_event(peer_id_1, peer_id_0, new_peer_notification, Inbound);
}
