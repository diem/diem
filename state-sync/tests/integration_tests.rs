// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_harness::{
    default_handler, PFN_NETWORK, VALIDATOR_NETWORK, VFN_NETWORK, VFN_NETWORK_2,
};
use anyhow::{bail, Result};
use diem_config::config::RoleType;
use diem_types::{transaction::TransactionListWithProof, waypoint::Waypoint};
use netcore::transport::ConnectionOrigin::*;
use network::protocols::direct_send::Message;
use state_sync::network::StateSyncMessage;
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
fn test_flaky_peer_sync() {
    let mut env = StateSyncEnvironment::new(2);

    env.start_validator_peer(0, false);

    // Create handler that causes error, but has successful retries
    let attempt = AtomicUsize::new(0);
    let handler = Box::new(move |resp| -> Result<TransactionListWithProof> {
        let fail_request = attempt.load(Ordering::Relaxed) == 0;
        attempt.fetch_add(1, Ordering::Relaxed);
        if fail_request {
            bail!("chunk fetch failed")
        } else {
            Ok(resp)
        }
    });
    env.start_state_sync_peer(
        1,
        handler,
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );

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

    let handler =
        Box::new(move |_| -> Result<TransactionListWithProof> { bail!("chunk fetch failed") });
    env.start_state_sync_peer(
        0,
        handler,
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.setup_state_sync_peer(
        1,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        100,
        300,
        false,
        None,
    );

    let validator_0 = env.get_state_sync_peer(0);
    let validator_1 = env.get_state_sync_peer(1);

    validator_0.commit(1);
    validator_1.sync_to(validator_0.latest_li());
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

    env.start_validator_peer(0, false);
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

    env.start_state_sync_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        waypoint,
        false,
        None,
    );
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
fn test_lagging_upstream_long_poll() {
    let mut env = StateSyncEnvironment::new(4);

    // Start 2 validators and 2 fullnodes
    env.start_validator_peer(0, true);
    env.start_validator_peer(1, true);
    env.setup_state_sync_peer(
        2,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        10_000,
        1_000_000,
        true,
        Some(vec![VFN_NETWORK.clone(), PFN_NETWORK.clone()]),
    );
    env.start_state_sync_peer(
        3,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        true,
        Some(vec![VFN_NETWORK.clone()]),
    );

    let validator_0 = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(2);
    let fullnode_1 = env.get_state_sync_peer(3);

    // Get peer ids of nodes (across different networks)
    let validator_0_peer_id = validator_0.get_peer_id(VALIDATOR_NETWORK.clone());
    let fullnode_0_peer_id_vfn = fullnode_0.get_peer_id(VFN_NETWORK.clone());
    let fullnode_0_peer_id_pfn = fullnode_0.get_peer_id(PFN_NETWORK.clone());
    let fullnode_1_peer_id_vfn = fullnode_1.get_peer_id(VFN_NETWORK.clone());
    let fullnode_1_peer_id_pfn = fullnode_1.get_peer_id(PFN_NETWORK.clone());

    // Commit version 400 at the validator
    validator_0.commit(400);
    drop(validator_0);
    drop(fullnode_0);
    drop(fullnode_1);

    // Validator and fullnode discover each other
    env.send_peer_event(fullnode_0_peer_id_vfn, validator_0_peer_id, true, Inbound);
    env.send_peer_event(validator_0_peer_id, fullnode_0_peer_id_vfn, true, Outbound);

    // Fullnodes discover each other
    env.send_peer_event(
        fullnode_0_peer_id_pfn,
        fullnode_1_peer_id_pfn,
        true,
        Inbound,
    );
    env.send_peer_event(
        fullnode_1_peer_id_pfn,
        fullnode_0_peer_id_pfn,
        true,
        Outbound,
    );

    // Deliver messages and verify versions and targets
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_vfn);
    check_chunk_request(message, 0, None);
    let (_, message) = env.deliver_msg(validator_0_peer_id);
    check_chunk_response(message, 400, 1, 250);
    env.get_state_sync_peer(2).wait_for_version(250, None);

    // Validator loses fullnode
    env.send_peer_event(fullnode_0_peer_id_vfn, validator_0_peer_id, false, Inbound);
    // Fullnode loses validator
    env.send_peer_event(validator_0_peer_id, fullnode_0_peer_id_vfn, false, Outbound);

    // Fullnode sends chunk request to other fullnode
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 250, None);

    // Validator 0 commits to new version and fullnode 1 is fast forwarded
    env.get_state_sync_peer(0).commit(500);
    env.clone_storage(0, 3);
    env.get_state_sync_peer(3).wait_for_version(500, Some(500));

    // Connect the validator and the failover fullnode so the fullnode can sync.
    // Validator discovers fullnode
    env.send_peer_event(fullnode_1_peer_id_vfn, validator_0_peer_id, true, Inbound);
    // Fullnode discovers validator
    env.send_peer_event(validator_0_peer_id, fullnode_1_peer_id_vfn, true, Outbound);

    // Trigger another commit so that the fullnodes's commit will trigger subscription delivery
    env.get_state_sync_peer(0).commit(600);
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_vfn);
    check_chunk_request(message, 500, None);
    let (_, message) = env.deliver_msg(validator_0_peer_id);
    check_chunk_response(message, 600, 501, 100);

    // Fullnode 1 sends long-poll subscription to fullnode 0
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 600, 251, 250);

    // Fullnode 0 sends chunk request to fullnode 1 and commits to latest state
    let (_, message) = env.deliver_msg(fullnode_0_peer_id_pfn);
    check_chunk_request(message, 500, None);
    let (_, message) = env.deliver_msg(fullnode_1_peer_id_pfn);
    check_chunk_response(message, 600, 501, 100);
    env.get_state_sync_peer(2).wait_for_version(600, Some(600));
}

#[test]
fn test_fullnode_catch_up_validator() {
    // Create validator and fullnode
    let mut env = StateSyncEnvironment::new(2);
    env.start_validator_peer(0, true);
    env.start_fullnode_peer(1, true);

    // Get peer ids of nodes
    let validator_peer_id = env
        .get_state_sync_peer(0)
        .get_peer_id(VALIDATOR_NETWORK.clone());
    let fullnode_peer_id = env.get_state_sync_peer(1).get_peer_id(VFN_NETWORK.clone());

    // Validator and fullnode discover each other.
    env.send_peer_event(fullnode_peer_id, validator_peer_id, true, Inbound);
    env.send_peer_event(validator_peer_id, fullnode_peer_id, true, Outbound);

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
        (1000, 0),
        (1250, 0),
        (1500, 0),
        (1750, 0),
        (2000, 0),
        (2250, 0),
        (2500, 0),
        (2750, 0),
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
#[ignore] // TODO: https://github.com/diem/diem/issues/5771
fn test_fn_failover() {
    let mut env = StateSyncEnvironment::new(5);

    env.start_validator_peer(0, true);
    env.setup_state_sync_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        60_000,
        true,
        Some(vec![VFN_NETWORK.clone(), PFN_NETWORK.clone()]),
    );

    // start up 3 publicly available VFN
    env.start_fullnode_peer(2, true);
    env.start_fullnode_peer(3, true);
    env.start_fullnode_peer(4, true);

    let validator = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(1);
    let fullnode_1 = env.get_state_sync_peer(2);
    let fullnode_2 = env.get_state_sync_peer(3);
    let fullnode_3 = env.get_state_sync_peer(4);

    // connect everyone
    let validator_peer_id = validator.get_peer_id(VFN_NETWORK.clone());
    let fn_0_vfn_peer_id = fullnode_0.get_peer_id(VFN_NETWORK.clone());
    let fn_0_public_peer_id = fullnode_0.get_peer_id(PFN_NETWORK.clone());
    let fn_1_peer_id = fullnode_1.get_peer_id(PFN_NETWORK.clone());
    let fn_2_peer_id = fullnode_1.get_peer_id(PFN_NETWORK.clone());
    let fn_3_peer_id = fullnode_3.get_peer_id(PFN_NETWORK.clone());

    drop(validator);
    drop(fullnode_0);
    drop(fullnode_1);
    drop(fullnode_2);
    drop(fullnode_3);

    // vfn network:
    // validator discovers fn_0
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, true, Inbound);
    // fn_0 discovers validator
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, true, Outbound);

    // public network:
    // fn_0 sends new peer event to all its upstream public peers
    let upstream_peer_ids = [fn_1_peer_id, fn_2_peer_id, fn_3_peer_id];
    for peer in upstream_peer_ids.iter() {
        env.send_peer_event(fn_0_public_peer_id, *peer, true, Inbound);
        env.send_peer_event(*peer, fn_0_public_peer_id, true, Outbound);
    }

    // commit some txns on v
    // check that fn_0 sends chunk requests to v only
    for num_commit in 1..=5 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            // we just directly sync up the storage of all the upstream peers of fn_0
            // for ease of testing
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        // deliver validator's chunk response
        if num_commit < 5 {
            env.deliver_msg(validator_peer_id);
        }
    }

    // bring down v
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, false, Inbound);
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, false, Outbound);

    // deliver chunk response to fn_0 after the lost peer event
    // so that the next chunk request is guaranteed to be sent after the lost peer event
    env.deliver_msg(validator_peer_id);

    // check that vfn sends chunk requests to the failover FNs only
    let mut last_fallback_recipient = None;
    for num_commit in 6..=10 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_public_peer_id);
        assert!(upstream_peer_ids.contains(&recipient));
        env.assert_no_message_sent(fn_0_vfn_peer_id);
        // deliver validator's chunk response
        if num_commit < 10 {
            let (chunk_response_recipient, _) = env.deliver_msg(recipient);
            assert_eq!(chunk_response_recipient, fn_0_public_peer_id);
        } else {
            last_fallback_recipient = Some(recipient);
        }
    }

    // bring down two public fallback
    // disconnect fn_1 and fn_0
    env.send_peer_event(fn_0_public_peer_id, fn_1_peer_id, false, Inbound);
    env.send_peer_event(fn_1_peer_id, fn_0_public_peer_id, false, Outbound);

    // disconnect fn_2 and fn_0
    env.send_peer_event(fn_0_public_peer_id, fn_2_peer_id, false, Inbound);
    env.send_peer_event(fn_2_peer_id, fn_0_public_peer_id, false, Outbound);

    // deliver chunk response to fn_0 after the lost peer events
    // so that the next chunk request is guaranteed to be sent after the lost peer events
    let (chunk_response_recipient, _) = env.deliver_msg(last_fallback_recipient.unwrap());
    assert_eq!(chunk_response_recipient, fn_0_public_peer_id);

    // check we only broadcast to the single live fallback peer (fn_3)
    for num_commit in 11..=15 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_public_peer_id);
        assert_eq!(recipient, fn_3_peer_id);
        env.assert_no_message_sent(fn_0_vfn_peer_id);
        // deliver validator's chunk response
        if num_commit < 15 {
            let (chunk_response_recipient, _) = env.deliver_msg(fn_3_peer_id);
            assert_eq!(chunk_response_recipient, fn_0_public_peer_id);
        }
    }

    // bring down everyone
    // disconnect fn_3 and fn_0
    env.send_peer_event(fn_3_peer_id, fn_0_public_peer_id, false, Outbound);
    env.send_peer_event(fn_0_public_peer_id, fn_3_peer_id, false, Inbound);

    // deliver chunk response to fn_0 after the lost peer events
    // so that the next chunk request is guaranteed to be sent after the lost peer events
    let (chunk_response_recipient, _) = env.deliver_msg(fn_3_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_public_peer_id);

    // check no sync requests are sent (all upstream are down)
    env.assert_no_message_sent(fn_0_vfn_peer_id);
    env.assert_no_message_sent(fn_0_public_peer_id);

    // bring back one fallback (fn_2)
    env.send_peer_event(fn_2_peer_id, fn_0_public_peer_id, true, Outbound);
    env.send_peer_event(fn_0_public_peer_id, fn_2_peer_id, true, Inbound);

    // check we only broadcast to the single live fallback peer (fn_2)
    for num_commit in 16..=20 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_public_peer_id);
        assert_eq!(recipient, fn_2_peer_id);
        env.assert_no_message_sent(fn_0_vfn_peer_id);
        // deliver validator's chunk response
        if num_commit < 20 {
            let (chunk_response_recipient, _) = env.deliver_msg(fn_2_peer_id);
            assert_eq!(chunk_response_recipient, fn_0_public_peer_id);
        }
    }

    // bring back v again
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, true, Inbound);
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, true, Outbound);

    let (chunk_response_recipient, _) = env.deliver_msg(fn_2_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_public_peer_id);

    // check that vfn sends chunk requests to v only, not fallback upstream
    for num_commit in 21..=25 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        if num_commit < 25 {
            // deliver validator's chunk response
            env.deliver_msg(validator_peer_id);
        }
    }

    // bring back all fallback
    let upstream_peers_to_revive = [
        env.get_state_sync_peer(2)
            .get_peer_id(VFN_NETWORK_2.clone()),
        env.get_state_sync_peer(4)
            .get_peer_id(VFN_NETWORK_2.clone()),
    ];
    for peer in upstream_peers_to_revive.iter() {
        env.send_peer_event(fn_0_public_peer_id, *peer, true, Inbound);
        env.send_peer_event(*peer, fn_0_public_peer_id, true, Outbound);
    }

    // deliver validator's chunk response after fallback peers are revived
    env.deliver_msg(validator_peer_id);

    // check that we only broadcast to v
    // check that vfn sends chunk requests to v only, not fallback upstream
    for num_commit in 26..=30 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        // deliver validator's chunk response
        env.deliver_msg(validator_peer_id);
    }
}

#[test]
#[ignore]
fn test_multicast_failover() {
    let mut env = StateSyncEnvironment::new(5);

    env.start_validator_peer(0, true);

    // set up node with more than 2 upstream networks, which is more than in standard prod setting
    // just to be safe
    let multicast_timeout_ms = 5_000;
    env.setup_state_sync_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        multicast_timeout_ms,
        true,
        Some(vec![
            VFN_NETWORK.clone(),
            VFN_NETWORK_2.clone(),
            PFN_NETWORK.clone(),
        ]),
    );

    // setup the other FN upstream peer
    env.start_fullnode_peer(2, true);
    env.start_fullnode_peer(3, true);
    env.start_fullnode_peer(4, true);

    let validator = env.get_state_sync_peer(0);
    let fullnode_0 = env.get_state_sync_peer(1);
    let fullnode_1 = env.get_state_sync_peer(2);
    let fullnode_2 = env.get_state_sync_peer(3);

    // connect everyone
    let validator_peer_id = validator.get_peer_id(VFN_NETWORK.clone());
    let fn_0_vfn_peer_id = fullnode_0.get_peer_id(VFN_NETWORK.clone());
    let fn_0_second_peer_id = fullnode_0.get_peer_id(VFN_NETWORK_2.clone());
    let fn_0_public_peer_id = fullnode_0.get_peer_id(PFN_NETWORK.clone());
    let fn_1_peer_id = fullnode_1.get_peer_id(VFN_NETWORK_2.clone());
    let fn_2_peer_id = fullnode_2.get_peer_id(PFN_NETWORK.clone());

    drop(validator);
    drop(fullnode_0);
    drop(fullnode_1);
    drop(fullnode_2);

    // vfn network:
    // validator discovers fn_0
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, true, Inbound);
    // fn_0 discovers validator
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, true, Outbound);

    // second private network: fn_1 is upstream to fn_0
    // fn_1 discovers fn_0
    env.send_peer_event(fn_0_second_peer_id, fn_1_peer_id, true, Inbound);
    // fn_0 discovers fn_1
    env.send_peer_event(fn_1_peer_id, fn_0_second_peer_id, true, Outbound);

    // public network: fn_2 is upstream to fn_1
    // fn_2 discovers fn_0
    env.send_peer_event(fn_0_public_peer_id, fn_2_peer_id, true, Inbound);
    // fn_0 discovers fn_2
    env.send_peer_event(fn_2_peer_id, fn_0_public_peer_id, true, Outbound);

    for num_commit in 1..=3 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_second_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        // deliver validator's chunk response
        if num_commit < 3 {
            env.deliver_msg(validator_peer_id);
        }
    }

    // we don't deliver the validator's last chunk response
    // wait for fn_0's chunk request to time out
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));

    // commit some with
    for num_commit in 4..=7 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        env.clone_storage(0, 2);
        env.get_state_sync_peer(2)
            .wait_for_version(num_commit * 5, None);

        // check that fn_0 sends chunk requests to both primary (vfn) and fallback ("second") network
        let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(primary, validator_peer_id);
        let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
        assert_eq!(secondary, fn_1_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);

        // deliver validator's chunk response
        if num_commit < 7 {
            env.deliver_msg(fn_1_peer_id);
        }
    }

    // we don't deliver the validator's or the secondary vfn network's last chunk response
    // wait for fn_0's chunk request to time out
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));

    for num_commit in 8..=11 {
        env.get_state_sync_peer(0).commit(num_commit * 5);
        env.clone_storage(0, 2);
        env.get_state_sync_peer(2)
            .wait_for_version(num_commit * 5, None);
        env.clone_storage(0, 3);
        env.get_state_sync_peer(3)
            .wait_for_version(num_commit * 5, None);

        // check that fn_0 sends chunk requests to both primary (vfn) and fallback ("second") network
        let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(primary, validator_peer_id);
        let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
        assert_eq!(secondary, fn_1_peer_id);
        let (public, _) = env.deliver_msg(fn_0_public_peer_id);
        assert_eq!(public, fn_2_peer_id);

        // deliver third fallback's chunk response
        env.deliver_msg(fn_2_peer_id);
    }

    // Test case: deliver chunks from all upstream with third fallback as first responder
    // Expected: next chunk request should still be sent to all upstream because validator did not deliver response first
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_1_peer_id);

    let mut num_commit = 12;
    env.get_state_sync_peer(0).commit(num_commit * 5);
    env.clone_storage(0, 2);
    env.get_state_sync_peer(2)
        .wait_for_version(num_commit * 5, None);
    env.clone_storage(0, 3);
    env.get_state_sync_peer(3)
        .wait_for_version(num_commit * 5, None);

    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    env.assert_no_message_sent(fn_0_vfn_peer_id);
    let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
    assert_eq!(secondary, fn_1_peer_id);
    let (public, _) = env.deliver_msg(fn_0_public_peer_id);
    assert_eq!(public, fn_2_peer_id);

    // Test case: deliver chunks from all upstream with secondary fallback as first responder
    // Expected: next chunk request should still be multicasted to all upstream because primary did not deliver response first
    env.deliver_msg(fn_1_peer_id);
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_2_peer_id);

    num_commit += 1;
    env.get_state_sync_peer(0).commit(num_commit * 5);
    env.clone_storage(0, 2);
    env.get_state_sync_peer(2)
        .wait_for_version(num_commit * 5, None);
    env.clone_storage(0, 3);
    env.get_state_sync_peer(3)
        .wait_for_version(num_commit * 5, None);

    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
    assert_eq!(secondary, fn_1_peer_id);
    let (public, _) = env.deliver_msg(fn_0_public_peer_id);
    assert_eq!(public, fn_2_peer_id);

    // Test case: deliver chunks from all upstream with primary as first responder
    // Expected: next chunk request should only be sent to primary network
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_1_peer_id);
    env.deliver_msg(fn_2_peer_id);

    num_commit += 1;
    env.get_state_sync_peer(0).commit(num_commit * 5);
    env.clone_storage(0, 2);
    env.get_state_sync_peer(2)
        .wait_for_version(num_commit * 5, None);
    env.clone_storage(0, 3);
    env.get_state_sync_peer(3)
        .wait_for_version(num_commit * 5, None);

    // because of optimistic chunk requesting, request will still be multicasted to all failover
    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
    assert_eq!(secondary, fn_1_peer_id);
    let (public, _) = env.deliver_msg(fn_0_public_peer_id);
    assert_eq!(public, fn_2_peer_id);

    // check that next chunk request is only be sent to primary network, i.e.
    // multicasting is over
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_1_peer_id);
    env.deliver_msg(fn_2_peer_id);

    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    env.assert_no_message_sent(fn_0_second_peer_id);
    env.assert_no_message_sent(fn_0_public_peer_id);
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
