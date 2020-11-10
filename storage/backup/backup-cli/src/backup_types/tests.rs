// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        state_snapshot::{
            backup::{StateSnapshotBackupController, StateSnapshotBackupOpt},
            restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        },
        transaction::{
            backup::{TransactionBackupController, TransactionBackupOpt},
            restore::{TransactionRestoreController, TransactionRestoreOpt},
        },
    },
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{
        backup_service_client::BackupServiceClient, test_utils::start_local_backup_service,
        GlobalBackupOpt, GlobalRestoreOpt, GlobalRestoreOptions,
    },
};
use executor_test_helpers::integration_test_impl::test_execution_with_storage_impl;
use libra_config::config::RocksdbConfig;
use libra_temppath::TempPath;
use libra_types::transaction::Version;
use libradb::LibraDB;
use proptest::prelude::*;
use std::{convert::TryInto, sync::Arc};
use storage_interface::DbReader;
use tokio::time::Duration;

#[derive(Debug)]
struct TestData {
    db: Arc<LibraDB>,
    txn_start_ver: Version,
    state_snapshot_ver: Option<Version>,
    target_ver: Version,
    latest_ver: Version,
}

fn test_data_strategy() -> impl Strategy<Value = TestData> {
    let db = test_execution_with_storage_impl();
    let latest_ver = db.get_latest_version().unwrap();

    (0..=latest_ver)
        .prop_flat_map(move |txn_start_ver| (Just(txn_start_ver), txn_start_ver..=latest_ver))
        .prop_flat_map(move |(txn_start_ver, state_snapshot_ver)| {
            (
                Just(txn_start_ver),
                prop_oneof![Just(Some(state_snapshot_ver)), Just(None)],
                state_snapshot_ver..=latest_ver,
            )
        })
        .prop_map(
            move |(txn_start_ver, state_snapshot_ver, target_ver)| TestData {
                db: Arc::clone(&db),
                txn_start_ver,
                state_snapshot_ver,
                target_ver,
                latest_ver,
            },
        )
}

fn test_end_to_end_impl(d: TestData) {
    let tgt_db_dir = TempPath::new();
    tgt_db_dir.create_as_dir().unwrap();
    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));
    let (mut rt, port) = start_local_backup_service(Arc::clone(&d.db));
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));
    let num_txns_to_backup = d.target_ver - d.txn_start_ver + 1;

    // Backup
    let global_backup_opt = GlobalBackupOpt {
        max_chunk_size: 2048,
    };
    let state_snapshot_manifest = d.state_snapshot_ver.map(|version| {
        rt.block_on(
            StateSnapshotBackupController::new(
                StateSnapshotBackupOpt { version },
                global_backup_opt.clone(),
                Arc::clone(&client),
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap()
    });
    let txn_manifest = rt
        .block_on(
            TransactionBackupController::new(
                TransactionBackupOpt {
                    start_version: d.txn_start_ver,
                    num_transactions: num_txns_to_backup as usize,
                },
                global_backup_opt,
                Arc::clone(&client),
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap();

    // Restore
    let global_restore_opt: GlobalRestoreOptions = GlobalRestoreOpt {
        dry_run: false,
        db_dir: Some(tgt_db_dir.path().to_path_buf()),
        target_version: Some(d.target_ver),
    }
    .try_into()
    .unwrap();
    if let Some(version) = d.state_snapshot_ver {
        rt.block_on(
            StateSnapshotRestoreController::new(
                StateSnapshotRestoreOpt {
                    manifest_handle: state_snapshot_manifest.unwrap(),
                    version,
                },
                global_restore_opt.clone(),
                Arc::clone(&store),
                None, /* epoch_history */
            )
            .run(),
        )
        .unwrap()
    }
    rt.block_on(
        TransactionRestoreController::new(
            TransactionRestoreOpt {
                manifest_handle: txn_manifest,
                replay_from_version: Some(
                    d.state_snapshot_ver.unwrap_or(Version::max_value() - 1) + 1,
                ),
            },
            global_restore_opt,
            store,
            None, /* epoch_history */
        )
        .run(),
    )
    .unwrap();

    // Check
    let tgt_db = LibraDB::open(
        &tgt_db_dir,
        false, /* read_only */
        None,  /* pruner */
        RocksdbConfig::default(),
    )
    .unwrap();
    assert_eq!(
        d.db.get_transactions(d.txn_start_ver, num_txns_to_backup, d.target_ver, false)
            .unwrap(),
        tgt_db
            .get_transactions(d.txn_start_ver, num_txns_to_backup, d.target_ver, false)
            .unwrap()
    );
    if let Some(state_snapshot_ver) = d.state_snapshot_ver {
        let first_replayed = state_snapshot_ver + 1;
        let num_replayed = d.target_ver - state_snapshot_ver;
        // Events recreated:
        assert_eq!(
            d.db.get_transactions(first_replayed, num_replayed, d.target_ver, true)
                .unwrap(),
            tgt_db
                .get_transactions(first_replayed, num_replayed, d.target_ver, true)
                .unwrap()
        );
    };

    rt.shutdown_timeout(Duration::from_secs(1));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_end_to_end(d in test_data_strategy()) {
        test_end_to_end_impl(d)
    }
}
