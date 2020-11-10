// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::transaction::{
        backup::{TransactionBackupController, TransactionBackupOpt},
        restore::{TransactionRestoreController, TransactionRestoreOpt},
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
use libra_types::transaction::Version;
use libradb::LibraDB;
use std::{convert::TryInto, mem::size_of, sync::Arc};
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

    let (mut rt, port) = start_local_backup_service(src_db);
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));

    let latest_version = blocks.last().unwrap().1.ledger_info().version();
    let total_txns = blocks.iter().fold(0, |x, b| x + b.0.len());
    assert_eq!(latest_version as usize + 1, total_txns);
    let txns = blocks
        .iter()
        .map(|(txns, _li)| txns)
        .flatten()
        .map(|txn_to_commit| txn_to_commit.transaction())
        .collect::<Vec<_>>();
    let max_chunk_size = txns
        .iter()
        .map(|t| lcs::to_bytes(t).unwrap().len())
        .max()
        .unwrap() // biggest txn
        + 115 // size of a serialized TransactionInfo
        + size_of::<u32>(); // record len header
    let first_ver_to_backup = (total_txns / 4) as Version;
    let num_txns_to_backup = total_txns - first_ver_to_backup as usize;
    let target_version = first_ver_to_backup + total_txns as Version / 2;
    let num_txns_to_restore = (target_version - first_ver_to_backup + 1) as usize;

    let manifest_handle = rt
        .block_on(
            TransactionBackupController::new(
                TransactionBackupOpt {
                    start_version: first_ver_to_backup,
                    num_transactions: num_txns_to_backup,
                },
                GlobalBackupOpt { max_chunk_size },
                client,
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap();

    rt.block_on(
        TransactionRestoreController::new(
            TransactionRestoreOpt {
                manifest_handle,
                replay_from_version: None, // max
            },
            GlobalRestoreOpt {
                dry_run: false,
                db_dir: Some(tgt_db_dir.path().to_path_buf()),
                target_version: Some(target_version),
            }
            .try_into()
            .unwrap(),
            store,
            None, /* epoch_history */
        )
        .run(),
    )
    .unwrap();

    // We don't write down any ledger infos when recovering transactions. State-sync needs to take
    // care of it before running consensus. The latest transactions are deemed "synced" instead of
    // "committed" most likely.
    let tgt_db = LibraDB::open(
        &tgt_db_dir,
        true, /* read_only */
        None, /* pruner */
        RocksdbConfig::default(),
    )
    .unwrap();
    assert_eq!(
        tgt_db
            .get_latest_transaction_info_option()
            .unwrap()
            .unwrap()
            .0,
        target_version,
    );
    let recovered_transactions = tgt_db
        .get_transactions(
            first_ver_to_backup,
            num_txns_to_restore as u64,
            target_version,
            false,
        )
        .unwrap()
        .transactions;

    assert_eq!(
        recovered_transactions,
        txns.into_iter()
            .skip(first_ver_to_backup as usize)
            .take(num_txns_to_restore)
            .cloned()
            .collect::<Vec<_>>()
    );

    rt.shutdown_timeout(Duration::from_secs(1));
}
