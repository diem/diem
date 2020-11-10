// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::epoch_ending::{
        backup::{EpochEndingBackupController, EpochEndingBackupOpt},
        restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
    },
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{
        backup_service_client::BackupServiceClient, test_utils::tmp_db_with_random_content,
        GlobalBackupOpt, GlobalRestoreOpt,
    },
};
use backup_service::start_backup_service;
use libra_config::{config::RocksdbConfig, utils::get_available_port};
use libra_temppath::TempPath;
use libradb::LibraDB;
use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use storage_interface::DbReader;
use tokio::time::Duration;

#[test]
fn end_to_end() {
    let (_src_db_dir, src_db, blocks) = tmp_db_with_random_content();
    let tgt_db_dir = TempPath::new();
    tgt_db_dir.create_as_dir().unwrap();

    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));

    let port = get_available_port();
    let mut rt = start_backup_service(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        src_db,
    );
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));

    let latest_epoch = blocks.last().unwrap().1.ledger_info().next_block_epoch();
    let target_version = blocks[blocks.len() / 2].1.ledger_info().version() + 1;
    let manifest_handle = rt
        .block_on(
            EpochEndingBackupController::new(
                EpochEndingBackupOpt {
                    start_epoch: 0,
                    end_epoch: latest_epoch,
                },
                GlobalBackupOpt {
                    max_chunk_size: 1024,
                },
                client,
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap();

    rt.block_on(
        EpochEndingRestoreController::new(
            EpochEndingRestoreOpt { manifest_handle },
            GlobalRestoreOpt {
                db_dir: Some(tgt_db_dir.path().to_path_buf()),
                dry_run: false,
                target_version: Some(target_version),
            }
            .try_into()
            .unwrap(),
            store,
            None,
        )
        .run(),
    )
    .unwrap();

    let expected_ledger_infos = blocks
        .into_iter()
        .map(|(_, li)| li)
        .filter(|li| li.ledger_info().ends_epoch() && li.ledger_info().version() <= target_version)
        .collect::<Vec<_>>();
    let target_version_next_block_epoch = expected_ledger_infos
        .last()
        .map(|li| li.ledger_info().next_block_epoch())
        .unwrap_or(0);

    let tgt_db = LibraDB::open(
        &tgt_db_dir,
        true, /* read_only */
        None, /* pruner */
        RocksdbConfig::default(),
    )
    .unwrap();
    assert_eq!(
        tgt_db
            .get_epoch_ending_ledger_infos(0, target_version_next_block_epoch)
            .unwrap()
            .ledger_info_with_sigs,
        expected_ledger_infos,
    );

    rt.shutdown_timeout(Duration::from_secs(1));
}
