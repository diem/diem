// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::new_local_swarm,
    test_utils::{create_and_fund_account, transfer_and_reconfig, transfer_coins},
};
use diem_config::config::{BootstrappingMode, ContinuousSyncingMode, NodeConfig};
use diem_rest_client::Client as RestClient;
use diem_sdk::{transaction_builder::TransactionFactory, types::LocalAccount};
use diem_types::PeerId;
use forge::{LocalSwarm, NodeExt, Swarm, SwarmExt};
use std::{
    fs,
    time::{Duration, Instant},
};

const MAX_CATCH_UP_SECS: u64 = 60; // The max time we'll wait for nodes to catch up

#[tokio::test]
async fn test_full_node_bootstrap_accounts() {
    // Create a validator swarm of 1 validator node
    let mut swarm = new_local_swarm(1).await;

    // Create a fullnode config that uses account state syncing
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.bootstrapping_mode =
        BootstrappingMode::DownloadLatestAccountStates;

    // Create (and stop) the fullnode
    let vfn_peer_id = create_full_node(vfn_config, &mut swarm).await;
    swarm.fullnode_mut(vfn_peer_id).unwrap().stop();

    // Enable account count support for the validator (with at most 2 accounts
    // per storage request).
    let validator = swarm.validators_mut().next().unwrap();
    let mut config = validator.config().clone();
    config.storage.account_count_migration = true;
    config
        .state_sync
        .storage_service
        .max_account_states_chunk_sizes = 2;
    config.save(validator.config_path()).unwrap();
    validator.restart().await.unwrap();
    validator
        .wait_until_healthy(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .await
        .unwrap();

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_peer_id, swarm, true).await;
}

#[tokio::test]
async fn test_full_node_bootstrap_outputs() {
    // Create a validator swarm of 1 validator node
    let mut swarm = new_local_swarm(1).await;

    // Create a fullnode config that uses transaction outputs to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.bootstrapping_mode =
        BootstrappingMode::ApplyTransactionOutputsFromGenesis;
    vfn_config
        .state_sync
        .state_sync_driver
        .continuous_syncing_mode = ContinuousSyncingMode::ApplyTransactionOutputs;

    // Create the fullnode
    let vfn_peer_id = create_full_node(vfn_config, &mut swarm).await;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_peer_id, swarm, true).await;
}

#[tokio::test]
async fn test_full_node_bootstrap_transactions() {
    // Create a validator swarm of 1 validator node
    let mut swarm = new_local_swarm(1).await;

    // Create a fullnode config that uses transactions to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.bootstrapping_mode =
        BootstrappingMode::ExecuteTransactionsFromGenesis;
    vfn_config
        .state_sync
        .state_sync_driver
        .continuous_syncing_mode = ContinuousSyncingMode::ExecuteTransactions;

    // Create the fullnode
    let vfn_peer_id = create_full_node(vfn_config, &mut swarm).await;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_peer_id, swarm, true).await;
}

#[tokio::test]
async fn test_full_node_continuous_sync_outputs() {
    // Create a validator swarm of 1 validator node
    let mut swarm = new_local_swarm(1).await;

    // Create a fullnode config that uses transaction outputs to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config
        .state_sync
        .state_sync_driver
        .continuous_syncing_mode = ContinuousSyncingMode::ApplyTransactionOutputs;

    // Create the fullnode
    let vfn_peer_id = create_full_node(vfn_config, &mut swarm).await;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_peer_id, swarm, false).await;
}

#[tokio::test]
async fn test_full_node_continuous_sync_transactions() {
    // Create a validator swarm of 1 validator node
    let mut swarm = new_local_swarm(1).await;

    // Create a fullnode config that uses transactions to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config
        .state_sync
        .state_sync_driver
        .continuous_syncing_mode = ContinuousSyncingMode::ExecuteTransactions;

    // Create the fullnode
    let vfn_peer_id = create_full_node(vfn_config, &mut swarm).await;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_peer_id, swarm, false).await;
}

/// Creates a new full node using the given config and swarm
async fn create_full_node(full_node_config: NodeConfig, swarm: &mut LocalSwarm) -> PeerId {
    // Create the fullnode
    let validator_peer_id = swarm.validators().next().unwrap().peer_id();
    let vfn_peer_id = swarm
        .add_validator_fullnode(
            &swarm.versions().max().unwrap(),
            full_node_config,
            validator_peer_id,
        )
        .await
        .unwrap();
    for fullnode in swarm.full_nodes_mut() {
        fullnode
            .wait_until_healthy(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
            .await
            .unwrap();
    }
    vfn_peer_id
}

/// A helper method that tests that a full node can sync from a validator after
/// a failure and continue to stay up-to-date.
async fn test_full_node_sync(vfn_peer_id: PeerId, mut swarm: LocalSwarm, epoch_changes: bool) {
    // Stop the fullnode
    swarm.fullnode_mut(vfn_peer_id).unwrap().stop();

    // Execute a number of transactions on the validator
    let validator_peer_id = swarm.validators().next().unwrap().peer_id();
    let validator_client = swarm.validator(validator_peer_id).unwrap().rest_client();
    let transaction_factory = swarm.chain_info().transaction_factory();

    let mut account_0 = create_and_fund_account(&mut swarm, 1000).await;
    let mut account_1 = create_and_fund_account(&mut swarm, 1000).await;
    execute_transactions(
        &mut swarm,
        &validator_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        epoch_changes,
    )
    .await;

    // Restart the fullnode and verify it can sync
    swarm
        .fullnode_mut(vfn_peer_id)
        .unwrap()
        .restart()
        .await
        .unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .await
        .unwrap();

    // Execute more transactions on the validator and verify the fullnode catches up
    execute_transactions(
        &mut swarm,
        &validator_client,
        &transaction_factory,
        &mut account_1,
        &account_0,
        epoch_changes,
    )
    .await;
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_validator_bootstrap_outputs() {
    // Create a swarm of 4 validators with state sync v2 enabled (output syncing)
    let mut swarm = new_local_swarm(4).await;
    for validator in swarm.validators_mut() {
        let mut config = validator.config().clone();
        config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
        config.state_sync.state_sync_driver.bootstrapping_mode =
            BootstrappingMode::ApplyTransactionOutputsFromGenesis;
        config.state_sync.state_sync_driver.continuous_syncing_mode =
            ContinuousSyncingMode::ApplyTransactionOutputs;
        config.save(validator.config_path()).unwrap();
        validator.restart().await.unwrap();
    }

    // Test the ability of the validators to sync
    test_validator_sync(swarm).await;
}

#[tokio::test]
async fn test_validator_bootstrap_transactions() {
    // Create a swarm of 4 validators with state sync v2 enabled (transaction syncing)
    let mut swarm = new_local_swarm(4).await;
    for validator in swarm.validators_mut() {
        let mut config = validator.config().clone();
        config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
        config.state_sync.state_sync_driver.bootstrapping_mode =
            BootstrappingMode::ExecuteTransactionsFromGenesis;
        config.state_sync.state_sync_driver.continuous_syncing_mode =
            ContinuousSyncingMode::ExecuteTransactions;
        config.save(validator.config_path()).unwrap();
        validator.restart().await.unwrap();
    }

    // Test the ability of the validators to sync
    test_validator_sync(swarm).await;
}

/// A helper method that tests that a validator can sync after a failure and
/// continue to stay up-to-date.
async fn test_validator_sync(mut swarm: LocalSwarm) {
    // Launch the swarm and wait for it to be ready
    swarm.launch().await.unwrap();

    // Execute multiple transactions through validator 0
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();
    let validator_client_0 = swarm
        .validator(validator_peer_ids[0])
        .unwrap()
        .rest_client();
    let transaction_factory = swarm.chain_info().transaction_factory();
    let mut account_0 = create_and_fund_account(&mut swarm, 1000).await;
    let mut account_1 = create_and_fund_account(&mut swarm, 1000).await;
    execute_transactions(
        &mut swarm,
        &validator_client_0,
        &transaction_factory,
        &mut account_0,
        &account_1,
        true,
    )
    .await;
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .await
        .unwrap();

    // Stop validator 1 and delete the storage
    let validator_1 = validator_peer_ids[1];
    swarm.validator_mut(validator_1).unwrap().stop();
    let node_config = swarm.validator_mut(validator_1).unwrap().config().clone();
    let state_db_path = node_config.storage.dir().join("diemdb");
    assert!(state_db_path.as_path().exists());
    fs::remove_dir_all(state_db_path).unwrap();

    // Execute more transactions
    execute_transactions(
        &mut swarm,
        &validator_client_0,
        &transaction_factory,
        &mut account_1,
        &account_0,
        true,
    )
    .await;

    // Restart validator 1 and wait for all nodes to catchup
    swarm.validator_mut(validator_1).unwrap().start().unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .await
        .unwrap();

    // Execute multiple transactions and verify validator 1 can sync
    execute_transactions(
        &mut swarm,
        &validator_client_0,
        &transaction_factory,
        &mut account_0,
        &account_1,
        true,
    )
    .await;
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_single_validator_failure() {
    // Create a swarm of 1 validator
    let mut swarm = new_local_swarm(1).await;
    swarm.launch().await.unwrap();

    // Enable state sync v2 and reboot the node
    let validator = swarm.validators_mut().next().unwrap();
    let mut config = validator.config().clone();
    config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    config.save(validator.config_path()).unwrap();
    validator.stop();
    swarm.launch().await.unwrap();

    // Execute multiple transactions
    let validator = swarm.validators_mut().next().unwrap();
    let validator_client = validator.rest_client();
    let transaction_factory = swarm.chain_info().transaction_factory();
    let mut account_0 = create_and_fund_account(&mut swarm, 1000).await;
    let mut account_1 = create_and_fund_account(&mut swarm, 1000).await;
    execute_transactions(
        &mut swarm,
        &validator_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        true,
    )
    .await;

    // Restart the validator
    let validator = swarm.validators_mut().next().unwrap();
    validator.stop();
    swarm.launch().await.unwrap();

    // Execute more transactions
    execute_transactions(
        &mut swarm,
        &validator_client,
        &transaction_factory,
        &mut account_1,
        &account_0,
        true,
    )
    .await;
}

/// Executes transactions using the given transaction factory, client and
/// accounts. If `force_epoch_changes` is true, also execute transactions to
/// force reconfigurations.
async fn execute_transactions(
    swarm: &mut LocalSwarm,
    client: &RestClient,
    transaction_factory: &TransactionFactory,
    sender: &mut LocalAccount,
    receiver: &LocalAccount,
    execute_epoch_changes: bool,
) {
    let num_transfers = 10;
    if execute_epoch_changes {
        transfer_and_reconfig(
            client,
            transaction_factory,
            swarm.chain_info().root_account,
            sender,
            receiver,
            num_transfers,
        )
        .await;
    } else {
        for _ in 0..num_transfers {
            // Execute simple transfer transactions
            transfer_coins(client, transaction_factory, sender, receiver, 1).await;
        }
    }
}
