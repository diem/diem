// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    test_utils::{
        compare_balances,
        diem_swarm_utils::{insert_waypoint, load_node_config, save_node_config},
        setup_swarm_and_client_proxy,
    },
    workspace_builder,
    workspace_builder::workspace_root,
};
use anyhow::{bail, Result};
use backup_cli::metadata::view::BackupStorageState;
use cli::client_proxy::ClientProxy;
use diem_temppath::TempPath;
use diem_types::transaction::Version;
use rand::random;
use std::{
    fs,
    path::Path,
    process::Command,
    string::ToString,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[test]
fn test_db_restore() {
    let (mut env, mut client) = setup_swarm_and_client_proxy(4, 1);

    // pre-build tools
    workspace_builder::get_bin("db-backup");
    workspace_builder::get_bin("db-restore");
    workspace_builder::get_bin("db-backup-verify");

    // set up: two accounts, a lot of money
    client.create_next_account(false).unwrap();
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mb", "0", "1000000", "XUS"], true)
        .unwrap();
    client
        .mint_coins(&["mb", "1", "1000000", "XUS"], true)
        .unwrap();
    client
        .transfer_coins(&["tb", "0", "1", "1", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(999999.0, "XUS".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(1000001.0, "XUS".to_string())],
        client.get_balances(&["b", "1"]).unwrap(),
    ));

    // start thread to transfer money from account 0 to account 1
    let accounts = client.copy_all_accounts();
    let transfer_quit = Arc::new(AtomicBool::new(false));
    let transfer_quit_clone = transfer_quit.clone();
    let transfer_thread = std::thread::spawn(|| {
        transfer_and_reconfig(client, 999999.0, 1000001.0, transfer_quit_clone)
    });

    // make a backup from node 1
    let (node1_config, _) = load_node_config(&env.validator_swarm, 1);
    let backup_path = db_backup(node1_config.storage.backup_service_address.port(), 1, 50);

    // take down node 0
    env.validator_swarm.kill_node(0);

    // nuke db
    let (mut node0_config, _) = load_node_config(&env.validator_swarm, 0);
    let genesis_waypoint = node0_config.base.waypoint.genesis_waypoint();
    insert_waypoint(&mut node0_config, genesis_waypoint);
    save_node_config(&mut node0_config, &env.validator_swarm, 0);
    let db_dir = node0_config.storage.dir();
    fs::remove_dir_all(db_dir.join("diemdb")).unwrap();
    fs::remove_dir_all(db_dir.join("consensusdb")).unwrap();

    // restore db from backup
    db_restore(backup_path.path(), db_dir.as_path());

    // start node 0 on top of restored db
    // (add_node() waits for it to connect to peers)
    env.validator_swarm.add_node(0).unwrap();

    // stop transferring
    transfer_quit.store(true, Ordering::Relaxed);
    let transferred = transfer_thread.join().unwrap();

    // verify it's caught up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());
    let mut client0 = env.get_validator_client(0, None);
    client0.set_accounts(accounts);
    assert!(compare_balances(
        vec![(999999.0 - transferred, "XUS".to_string())],
        client0.get_balances(&["b", "0"]).unwrap(),
    ));
}

fn db_backup_verify(backup_path: &Path) {
    let now = Instant::now();
    let bin_path = workspace_builder::get_bin("db-backup-verify");
    let metadata_cache_path = TempPath::new();

    metadata_cache_path.create_as_dir().unwrap();

    let output = Command::new(bin_path.as_path())
        .current_dir(workspace_root())
        .args(&[
            "--metadata-cache-dir",
            metadata_cache_path.path().to_str().unwrap(),
            "local-fs",
            "--dir",
            backup_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("db-backup-verify failed, output: {:?}", output);
    }
    println!("Backup verified in {} seconds.", now.elapsed().as_secs());
}

fn wait_for_backups(
    target_epoch: u64,
    target_version: u64,
    now: Instant,
    bin_path: &Path,
    metadata_cache_path: &Path,
    backup_path: &Path,
) -> Result<()> {
    for _ in 0..60 {
        // the verify should always succeed.
        db_backup_verify(backup_path);

        let output = Command::new(bin_path)
            .current_dir(workspace_root())
            .args(&[
                "one-shot",
                "query",
                "backup-storage-state",
                "--metadata-cache-dir",
                metadata_cache_path.to_str().unwrap(),
                "local-fs",
                "--dir",
                backup_path.to_str().unwrap(),
            ])
            .output()?
            .stdout;
        let state: BackupStorageState = std::str::from_utf8(&output)?.parse()?;
        if state.latest_epoch_ending_epoch.is_some()
            && state.latest_transaction_version.is_some()
            && state.latest_state_snapshot_version.is_some()
            && state.latest_epoch_ending_epoch.unwrap() >= target_epoch
            && state.latest_transaction_version.unwrap() >= target_version
        {
            println!("Backup created in {} seconds.", now.elapsed().as_secs());
            return Ok(());
        }
        println!("Backup storage state: {}", state);
        std::thread::sleep(Duration::from_secs(1));
    }

    bail!("Failed to create backup.");
}

fn db_backup(backup_service_port: u16, target_epoch: u64, target_version: Version) -> TempPath {
    let now = Instant::now();
    let bin_path = workspace_builder::get_bin("db-backup");
    let metadata_cache_path1 = TempPath::new();
    let metadata_cache_path2 = TempPath::new();
    let backup_path = TempPath::new();

    metadata_cache_path1.create_as_dir().unwrap();
    metadata_cache_path2.create_as_dir().unwrap();
    backup_path.create_as_dir().unwrap();

    // spawn the backup coordinator
    let mut backup_coordinator = Command::new(bin_path.as_path())
        .current_dir(workspace_root())
        .args(&[
            "coordinator",
            "run",
            "--backup-service-address",
            &format!("http://localhost:{}", backup_service_port),
            "--transaction-batch-size",
            "20",
            "--state-snapshot-interval",
            "40",
            "--metadata-cache-dir",
            metadata_cache_path1.path().to_str().unwrap(),
            "local-fs",
            "--dir",
            backup_path.path().to_str().unwrap(),
        ])
        .spawn()
        .unwrap();

    // watch the backup storage, wait for it to reach target epoch and version
    let wait_res = wait_for_backups(
        target_epoch,
        target_version,
        now,
        bin_path.as_path(),
        metadata_cache_path2.path(),
        backup_path.path(),
    );
    backup_coordinator.kill().unwrap();
    wait_res.unwrap();
    backup_path
}

fn db_restore(backup_path: &Path, db_path: &Path) {
    let now = Instant::now();
    let bin_path = workspace_builder::get_bin("db-restore");
    let metadata_cache_path = TempPath::new();

    metadata_cache_path.create_as_dir().unwrap();

    let output = Command::new(bin_path.as_path())
        .current_dir(workspace_root())
        .args(&[
            "--target-db-dir",
            db_path.to_str().unwrap(),
            "auto",
            "--metadata-cache-dir",
            metadata_cache_path.path().to_str().unwrap(),
            "local-fs",
            "--dir",
            backup_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("db-restore failed, output: {:?}", output);
    }
    println!("Backup restored in {} seconds.", now.elapsed().as_secs());
}

fn transfer_and_reconfig(
    mut client: ClientProxy,
    balance0: f64,
    balance1: f64,
    quit: Arc<AtomicBool>,
) -> f64 {
    let now = Instant::now();
    let mut transferred = 0f64;
    while !quit.load(Ordering::Relaxed) {
        if random::<u16>() % 10 == 0 {
            println!(
                "Changing diem version to {}: {:?}",
                transferred,
                client
                    .change_diem_version(&["change_diem_version", &transferred.to_string()], true)
            );
        }

        client
            .transfer_coins(&["tb", "0", "1", "1", "XUS"], true)
            .unwrap();
        transferred += 1.0;

        assert!(compare_balances(
            vec![(balance0 - transferred, "XUS".to_string())],
            client.get_balances(&["b", "0"]).unwrap(),
        ));
        assert!(compare_balances(
            vec![(balance1 + transferred, "XUS".to_string())],
            client.get_balances(&["b", "1"]).unwrap(),
        ));
    }
    println!(
        "{} coins transfered in {} seconds.",
        transferred,
        now.elapsed().as_secs()
    );
    transferred
}
