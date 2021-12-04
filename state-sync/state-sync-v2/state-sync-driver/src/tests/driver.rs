// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tests::{
    utils,
    utils::{create_ledger_info_at_version, create_test_transaction},
};
use claim::{assert_err, assert_ok, assert_some};
use consensus_notifications::ConsensusNotificationSender;
use futures::{FutureExt, StreamExt};

#[tokio::test]
async fn test_bootstrap_notification() {
    // Create a driver for a validator with a waypoint at version 0
    let (_validator_driver, consensus_notifier, _, _) = utils::create_validator_driver();

    // Send a new commit notification to the driver to complete bootstrapping
    let result = consensus_notifier
        .notify_new_commit(vec![create_test_transaction()], vec![])
        .await;
    assert_ok!(result);

    // TODO(joshlind): verify the client is notified (the node has already bootstrapped)

    // TODO(joshlind): verify listeners are also eventually notified when the node bootstraps
}

#[tokio::test]
async fn test_consensus_commit_notification() {
    // Create a driver for a full node
    let (_full_node_driver, consensus_notifier, _, _) = utils::create_full_node_driver();

    // Verify that full nodes can't process commit notifications
    let result = consensus_notifier
        .notify_new_commit(vec![create_test_transaction()], vec![])
        .await;
    assert_err!(result);

    // Create a driver for a validator with a waypoint at version 0
    let (_validator_driver, consensus_notifier, mut mempool_listener, mut reconfig_listener) =
        utils::create_validator_driver();

    // Send a new commit notification to the driver
    let result = consensus_notifier
        .notify_new_commit(vec![create_test_transaction()], vec![])
        .await;
    assert_ok!(result);

    // Verify that mempool is notified
    let mempool_commit_notification = mempool_listener.select_next_some().now_or_never().unwrap();
    let result = mempool_listener.ack_commit_notification(mempool_commit_notification);
    assert_ok!(result);

    // Verify reconfiguration notifications are published
    assert_some!(reconfig_listener
        .notification_receiver
        .select_next_some()
        .now_or_never());
}

#[tokio::test]
async fn test_consensus_sync_request() {
    // Create a driver for a full node
    let (_full_node_driver, consensus_notifier, _, _) = utils::create_full_node_driver();

    // Verify that full nodes can't process sync requests
    let result = consensus_notifier
        .sync_to_target(create_ledger_info_at_version(0))
        .await;
    assert_err!(result);

    // Create a driver for a validator with a waypoint at version 0
    let (_validator_driver, consensus_notifier, _, _) = utils::create_validator_driver();

    // Send a new commit notification to the driver to complete bootstrapping
    let result = consensus_notifier
        .notify_new_commit(vec![create_test_transaction()], vec![])
        .await;
    assert_ok!(result);

    // Verify the driver notifies consensus that it has already reached the target
    let result = consensus_notifier
        .sync_to_target(create_ledger_info_at_version(0))
        .await;
    assert_ok!(result);

    // TODO(joshlind): verify the validator sync request should error because the node has not bootstrapped

    // TODO(joshlind): update the database and ensure that consensus is eventually notified
}
