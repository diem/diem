// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::new_local_swarm,
    storage::{db_backup, db_restore},
    test_utils::{
        assert_balance, create_and_fund_account,
        diem_swarm_utils::{create_root_storage, insert_waypoint},
        transfer_coins,
    },
    workspace_builder,
    workspace_builder::workspace_root,
};
use anyhow::anyhow;
use diem_operational_tool::test_helper::OperationalTool;
use diem_temppath::TempPath;
use diem_transaction_builder::stdlib::encode_remove_validator_and_reconfigure_script;
use diem_types::{
    account_config::diem_root_address,
    epoch_change::EpochChangeProof,
    transaction::{Transaction, WriteSetPayload},
    waypoint::Waypoint,
};
use forge::{Node, NodeExt, Swarm, SwarmExt};
use regex::Regex;
use std::{
    fs,
    fs::File,
    io::Write,
    path::PathBuf,
    process::Command,
    str::FromStr,
    thread,
    thread::sleep,
    time::{Duration, Instant},
};

#[test]
/// This test verifies the flow of a genesis transaction after the chain starts.
/// 1. Test the consensus sync_only mode, every node should stop at the same version.
/// 2. Test the db-bootstrapper applying a manual genesis transaction (remove validator 0) on diemdb directly
/// 3. Test the nodes and clients resume working after updating waypoint
/// 4. Test a node lagging behind can sync to the waypoint
fn test_genesis_transaction_flow() {
    // prebuild tools.
    let db_bootstrapper = workspace_builder::get_bin("db-bootstrapper");
    workspace_builder::get_bin("db-backup");
    workspace_builder::get_bin("db-restore");
    workspace_builder::get_bin("db-backup-verify");

    let mut swarm = new_local_swarm(4);
    let chain_id = swarm.chain_id();
    let transaction_factory = swarm.chain_info().transaction_factory();
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();

    println!("1. Set sync_only = true for the first node and check it can sync to others");
    let node_to_kill = validator_peer_ids[3];
    let node_config_path = swarm.validator(node_to_kill).unwrap().config_path();
    let mut node_config = swarm.validator(node_to_kill).unwrap().config().clone();
    node_config.consensus.sync_only = true;
    node_config.save(&node_config_path).unwrap();

    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .restart()
        .unwrap();
    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();

    let mut account_0 = create_and_fund_account(&mut swarm, 10);
    let account_1 = create_and_fund_account(&mut swarm, 10);

    println!("2. Set sync_only = true for all nodes and restart");
    for validator in swarm.validators_mut() {
        let mut node_config = validator.config().clone();
        node_config.consensus.sync_only = true;
        node_config.save(validator.config_path()).unwrap();
        validator.restart().unwrap();
        validator
            .wait_until_healthy(Instant::now() + Duration::from_secs(10))
            .unwrap();
    }

    println!("3. delete one node's db and test they can still sync when sync_only is true for every nodes");
    swarm.validator_mut(node_to_kill).unwrap().stop();
    fs::remove_dir_all(node_config.storage.dir()).unwrap();
    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .restart()
        .unwrap();
    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();

    println!("4. verify all nodes are at the same round and no progress being made in 5 sec");
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();

    println!("Wait for node to synchronize on same version.");
    thread::sleep(Duration::from_secs(1));

    let mut known_version = None;
    for i in 0..5 {
        for validator in swarm.validators() {
            let version = validator
                .json_rpc_client()
                .get_metadata()
                .unwrap()
                .into_inner()
                .version;
            match known_version {
                Some(v) if v != version => panic!(
                    "version not equal, last known: {}, node {} is {}",
                    v,
                    validator.name(),
                    version,
                ),
                None => known_version = Some(version),
                _ => continue,
            }
        }
        println!(
            "The last know round after {} sec is {}",
            i,
            known_version.unwrap()
        );
        sleep(Duration::from_secs(1));
    }

    println!("5. kill all nodes and prepare a genesis txn to remove validator 0");
    let validator_address = node_config.validator_network.as_ref().unwrap().peer_id();
    let op_tool = OperationalTool::new(
        swarm
            .validator(node_to_kill)
            .unwrap()
            .json_rpc_endpoint()
            .to_string(),
        chain_id,
    );
    let diem_root = create_root_storage(&mut swarm);
    let config = op_tool
        .validator_config(validator_address, Some(&diem_root))
        .unwrap();
    let name = config.name.as_bytes().to_vec();

    for validator in swarm.validators_mut() {
        validator.stop()
    }
    let genesis_transaction = Transaction::GenesisTransaction(WriteSetPayload::Script {
        execute_as: diem_root_address(),
        script: encode_remove_validator_and_reconfigure_script(0, name, validator_address),
    });
    let genesis_path = TempPath::new();
    genesis_path.create_as_file().unwrap();
    let mut file = File::create(genesis_path.path()).unwrap();
    file.write_all(&bcs::to_bytes(&genesis_transaction).unwrap())
        .unwrap();

    println!("6. prepare the waypoint with the transaction");
    let waypoint_command = Command::new(db_bootstrapper.as_path())
        .current_dir(workspace_root())
        .args(&vec![
            node_config.storage.dir().to_str().unwrap(),
            "--genesis-txn-file",
            genesis_path.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let output = std::str::from_utf8(&waypoint_command.stdout).unwrap();
    let waypoint = parse_waypoint(output);

    println!("7. apply genesis transaction for nodes 1, 2, 3");
    for validator in swarm
        .validators_mut()
        .filter(|v| v.peer_id() != node_to_kill)
    {
        let mut node_config = validator.config().clone();
        insert_waypoint(&mut node_config, waypoint);
        node_config.execution.genesis = Some(genesis_transaction.clone());
        // reset the sync_only flag to false
        node_config.consensus.sync_only = false;
        node_config.save(validator.config_path()).unwrap();
        validator.start().unwrap();
        validator
            .wait_until_healthy(Instant::now() + Duration::from_secs(10))
            .unwrap();
    }

    println!("8. verify it's able to mint after the waypoint");
    let client_0 = swarm
        .validator(validator_peer_ids[0])
        .unwrap()
        .json_rpc_client();
    transfer_coins(
        &client_0,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );
    assert_balance(&client_0, &account_0, 9);
    assert_balance(&client_0, &account_1, 11);

    // Create a new epoch to make things more complicated
    let txn = swarm
        .chain_info()
        .root_account
        .sign_with_transaction_builder(transaction_factory.update_diem_version(0, 12345));
    client_0.submit(&txn).unwrap();
    client_0
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();

    // Make full DB backup for later use. The backup crosses the new genesis.
    let state_proof = client_0.get_state_proof(0).unwrap();
    let version = state_proof.state().version;
    let epoch_change_proof: EpochChangeProof =
        bcs::from_bytes(state_proof.inner().epoch_change_proof.inner()).unwrap();

    let epoch = epoch_change_proof
        .ledger_info_with_sigs
        .last()
        .unwrap()
        .ledger_info()
        .next_block_epoch();
    let backup_path = db_backup(
        swarm
            .validator(validator_peer_ids[0])
            .unwrap()
            .config()
            .storage
            .backup_service_address
            .port(),
        epoch.checked_sub(1).unwrap(), // target epoch: most recently closed epoch
        version,                       // target version
        version as usize,              // txn batch size (version 0 is in its own batch)
        version as usize,              // state snapshot interval
        &[waypoint],
    );

    println!("9. add node 0 back and test if it can sync to the waypoint via state synchronizer");
    let op_tool = OperationalTool::new(
        swarm
            .validator(validator_peer_ids[0])
            .unwrap()
            .json_rpc_endpoint()
            .to_string(),
        chain_id,
    );
    let _ = op_tool
        .add_validator(validator_address, &diem_root, false)
        .unwrap();

    // setup the waypoint for node 0
    node_config.execution.genesis = None;
    node_config.execution.genesis_file_location = PathBuf::from("");
    insert_waypoint(&mut node_config, waypoint);
    node_config.save(node_config_path).unwrap();
    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .restart()
        .unwrap();
    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();

    let client = swarm.validator(node_to_kill).unwrap().json_rpc_client();
    transfer_coins(&client, &transaction_factory, &mut account_0, &account_1, 1);
    assert_balance(&client_0, &account_0, 8);
    assert_balance(&client_0, &account_1, 12);

    println!("10. nuke DB on node 0, and run db-restore, test if it rejoins the network okay.");
    swarm.validator_mut(node_to_kill).unwrap().stop();

    let db_dir = node_config.storage.dir();
    fs::remove_dir_all(&db_dir).unwrap();
    db_restore(backup_path.path(), db_dir.as_path(), &[waypoint]);

    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .restart()
        .unwrap();
    swarm
        .validator_mut(node_to_kill)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();

    transfer_coins(&client, &transaction_factory, &mut account_0, &account_1, 1);
    assert_balance(&client_0, &account_0, 7);
    assert_balance(&client_0, &account_1, 13);
}

fn parse_waypoint(db_bootstrapper_output: &str) -> Waypoint {
    let waypoint = Regex::new(r"Got waypoint: (\d+:\w+)")
        .unwrap()
        .captures(db_bootstrapper_output)
        .ok_or_else(|| anyhow!("Failed to parse db-bootstrapper output."));
    Waypoint::from_str(waypoint.unwrap()[1].into()).unwrap()
}
