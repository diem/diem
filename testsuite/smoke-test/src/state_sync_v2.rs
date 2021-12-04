// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    scripts_and_modules::enable_open_publishing,
    smoke_test_environment::new_local_swarm,
    test_utils::{assert_balance, create_and_fund_account, transfer_coins},
};
use diem_config::config::{BootstrappingMode, NodeConfig, SyncingMode};
use diem_sdk::{
    client::BlockingClient, transaction_builder::TransactionFactory, types::LocalAccount,
};
use forge::{LocalSwarm, NodeExt, Swarm, SwarmExt};
use std::time::{Duration, Instant};

const MAX_CATCH_UP_SECS: u64 = 60; // The max time we'll wait for nodes to catch up

#[test]
fn test_full_node_bootstrap_outputs() {
    // Create a validator swarm of 1 validator node
    let swarm = new_local_swarm(1);

    // Create a fullnode config that uses transactions to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.bootstrapping_mode =
        BootstrappingMode::ApplyTransactionOutputsFromGenesis;
    vfn_config.state_sync.state_sync_driver.syncing_mode = SyncingMode::ApplyTransactionOutputs;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_config, swarm, true)
}

#[test]
fn test_full_node_bootstrap_transactions() {
    // Create a validator swarm of 1 validator node
    let swarm = new_local_swarm(1);

    // Create a fullnode config that uses transaction outputs to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.bootstrapping_mode =
        BootstrappingMode::ExecuteTransactionsFromGenesis;
    vfn_config.state_sync.state_sync_driver.syncing_mode = SyncingMode::ExecuteTransactions;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_config, swarm, true)
}

#[test]
fn test_full_node_continuous_sync_outputs() {
    // Create a validator swarm of 1 validator node
    let swarm = new_local_swarm(1);

    // Create a fullnode config that uses transactions to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.syncing_mode = SyncingMode::ApplyTransactionOutputs;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_config, swarm, false)
}

#[test]
fn test_full_node_continuous_sync_transactions() {
    // Create a validator swarm of 1 validator node
    let swarm = new_local_swarm(1);

    // Create a fullnode config that uses transaction outputs to sync
    let mut vfn_config = NodeConfig::default_for_validator_full_node();
    vfn_config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
    vfn_config.state_sync.state_sync_driver.syncing_mode = SyncingMode::ExecuteTransactions;

    // Test the ability of the fullnode to sync
    test_full_node_sync(vfn_config, swarm, false)
}

/// A helper method that tests that a full node can sync from a validator after
/// a failure and continue to stay up-to-date.
fn test_full_node_sync(full_node_config: NodeConfig, mut swarm: LocalSwarm, epoch_changes: bool) {
    // Create the fullnode
    let validator_peer_id = swarm.validators().next().unwrap().peer_id();
    let vfn_peer_id = swarm
        .add_validator_fullnode(
            &swarm.versions().max().unwrap(),
            full_node_config,
            validator_peer_id,
        )
        .unwrap();

    // Start the validator and fullnode (make sure they boot up)
    swarm
        .validator_mut(validator_peer_id)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();
    for fullnode in swarm.full_nodes_mut() {
        fullnode
            .wait_until_healthy(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
            .unwrap();
    }

    // Stop the fullnode
    swarm.fullnode_mut(vfn_peer_id).unwrap().stop();

    // Execute a number of transactions on the validator
    let validator_client = swarm
        .validator(validator_peer_id)
        .unwrap()
        .json_rpc_client();
    let transaction_factory = swarm.chain_info().transaction_factory();
    let mut account_0 = create_and_fund_account(&mut swarm, 1000);
    let mut account_1 = create_and_fund_account(&mut swarm, 1000);
    execute_transactions(
        &mut swarm,
        &validator_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        epoch_changes,
    );

    // Restart the fullnode and verify it can sync
    swarm.fullnode_mut(vfn_peer_id).unwrap().restart().unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();

    // Execute more transactions on the validator and verify the fullnode catches up
    execute_transactions(
        &mut swarm,
        &validator_client,
        &transaction_factory,
        &mut account_1,
        &account_0,
        epoch_changes,
    );
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();
}

/// Executes transactions using the given transaction factory, client and
/// accounts. If `epoch_changes` is true, also executes transactions to force
/// reconfigurations.
fn execute_transactions(
    swarm: &mut LocalSwarm,
    json_rpc_client: &BlockingClient,
    transaction_factory: &TransactionFactory,
    account_0: &mut LocalAccount,
    account_1: &LocalAccount,
    epoch_changes: bool,
) {
    for _ in 0..10 {
        // Execute simple transfer transactions
        transfer_coins(
            json_rpc_client,
            transaction_factory,
            account_0,
            account_1,
            1,
        );

        // Cause an epoch change if required
        if epoch_changes {
            let root_account = swarm.chain_info().root_account;
            enable_open_publishing(json_rpc_client, transaction_factory, root_account).unwrap();
        }
    }
}

#[test]
fn test_basic_validator_restart_transactions() {
    // Create a swarm of 4 validators with state sync v2 enabled (transaction syncing)
    let mut swarm = new_local_swarm(4);
    for validator in swarm.validators_mut() {
        let mut config = validator.config().clone();
        config.state_sync.state_sync_driver.enable_state_sync_v2 = true;
        config.state_sync.state_sync_driver.bootstrapping_mode =
            BootstrappingMode::ExecuteTransactionsFromGenesis;
        config.state_sync.state_sync_driver.syncing_mode = SyncingMode::ExecuteTransactions;
        config.save(validator.config_path()).unwrap();
        validator.restart().unwrap();
    }
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();

    // Wait for the swarm to be ready
    swarm.launch().unwrap();

    // Execute a single transaction through validator 0
    let validator_client_0 = swarm
        .validator(validator_peer_ids[0])
        .unwrap()
        .json_rpc_client();
    let transaction_factory = swarm.chain_info().transaction_factory();
    let mut account_0 = create_and_fund_account(&mut swarm, 1000);
    let mut account_1 = create_and_fund_account(&mut swarm, 1000);
    transfer_coins(
        &validator_client_0,
        &transaction_factory,
        &mut account_0,
        &account_1,
        10,
    );
    assert_balance(&validator_client_0, &account_0, 990);
    assert_balance(&validator_client_0, &account_1, 1010);
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();

    // Stop validator 1
    let validator_1 = validator_peer_ids[1];
    swarm.validator_mut(validator_1).unwrap().stop();

    // Execute another transaction and verify consensus is still operating
    transfer_coins(
        &validator_client_0,
        &transaction_factory,
        &mut account_1,
        &account_0,
        5,
    );
    assert_balance(&validator_client_0, &account_0, 995);
    assert_balance(&validator_client_0, &account_1, 1005);

    // Restart validator 1 and wait for all nodes to catchup
    swarm.validator_mut(validator_1).unwrap().start().unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();

    // Connect to validator 1 and verify state
    let validator_client_1 = swarm.validator(validator_1).unwrap().json_rpc_client();
    assert_balance(&validator_client_1, &account_0, 995);
    assert_balance(&validator_client_1, &account_1, 1005);

    // Test multiple transactions
    for _ in 0..10 {
        transfer_coins(
            &validator_client_0,
            &transaction_factory,
            &mut account_1,
            &account_0,
            1,
        );
    }
    assert_balance(&validator_client_0, &account_0, 1005);
    assert_balance(&validator_client_0, &account_1, 995);
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();

    // Stop validator 1 and send multiple transactions
    swarm.validator_mut(validator_1).unwrap().stop();
    for _ in 0..10 {
        transfer_coins(
            &validator_client_0,
            &transaction_factory,
            &mut account_0,
            &account_1,
            10,
        );
    }

    // Restart validator 1 and wait for all nodes to catchup
    swarm.validator_mut(validator_1).unwrap().start().unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(MAX_CATCH_UP_SECS))
        .unwrap();
    assert_balance(&validator_client_1, &account_0, 905);
    assert_balance(&validator_client_1, &account_1, 1095);
}
