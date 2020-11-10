// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::state_snapshot::{
        backup::{StateSnapshotBackupController, StateSnapshotBackupOpt},
        restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
    },
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{
        backup_service_client::BackupServiceClient,
        test_utils::{start_local_backup_service, tmp_db_with_random_content},
        GlobalBackupOpt, GlobalRestoreOpt,
    },
};
use libra_config::config::RocksdbConfig;
use libra_temppath::TempPath;
use libra_types::transaction::PRE_GENESIS_VERSION;
use libradb::LibraDB;
use std::{convert::TryInto, sync::Arc};
use storage_interface::DbReader;
use tokio::time::Duration;

#[test]
fn end_to_end() {
    let (_src_db_dir, src_db, _blocks) = tmp_db_with_random_content();
    let tgt_db_dir = TempPath::new();
    tgt_db_dir.create_as_dir().unwrap();
    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));

    let latest_tree_state = src_db.get_latest_tree_state().unwrap();
    let version = latest_tree_state.num_transactions - 1;
    let state_root_hash = latest_tree_state.account_state_root_hash;

    let (mut rt, port) = start_local_backup_service(src_db);
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));

    let manifest_handle = rt
        .block_on(
            StateSnapshotBackupController::new(
                StateSnapshotBackupOpt { version },
                GlobalBackupOpt {
                    max_chunk_size: 500,
                },
                client,
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap();

    rt.block_on(
        StateSnapshotRestoreController::new(
            StateSnapshotRestoreOpt {
                manifest_handle,
                version: PRE_GENESIS_VERSION,
            },
            GlobalRestoreOpt {
                dry_run: false,
                db_dir: Some(tgt_db_dir.path().to_path_buf()),
                target_version: None, // max
            }
            .try_into()
            .unwrap(),
            store,
            None, /* epoch_history */
        )
        .run(),
    )
    .unwrap();

    let tgt_db = LibraDB::open(
        &tgt_db_dir,
        true, /* read_only */
        None, /* pruner */
        RocksdbConfig::default(),
    )
    .unwrap();
    assert_eq!(
        tgt_db
            .get_latest_tree_state()
            .unwrap()
            .account_state_root_hash,
        state_root_hash,
    );

    rt.shutdown_timeout(Duration::from_secs(1));
}
