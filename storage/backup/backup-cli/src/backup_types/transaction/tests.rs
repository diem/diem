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
        test_utils::{tmp_db_empty, tmp_db_with_random_content},
        GlobalBackupOpt,
    },
};
use backup_service::start_backup_service;
use libra_config::utils::get_available_port;
use libra_temppath::TempPath;
use libra_types::transaction::Version;
use std::{mem::size_of, sync::Arc};
use storage_interface::DbReader;
use tokio::time::Duration;

#[test]
fn end_to_end() {
    let (_src_db_dir, src_db, blocks) = tmp_db_with_random_content();
    let (_tgt_db_dir, tgt_db) = tmp_db_empty();
    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));

    let port = get_available_port();
    let mut rt = start_backup_service(port, src_db);
    let client = Arc::new(BackupServiceClient::new(port));

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
    let first_ver_to_backup = (total_txns / 2) as Version;
    let num_txns_to_backup = total_txns - first_ver_to_backup as usize;

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
            TransactionRestoreOpt { manifest_handle },
            store,
            Arc::new(tgt_db.get_restore_handler()),
        )
        .run(),
    )
    .unwrap();

    let recovered_transactions = tgt_db
        .get_transactions(
            first_ver_to_backup,
            num_txns_to_backup as u64,
            latest_version,
            false,
        )
        .unwrap()
        .transactions;

    assert_eq!(
        recovered_transactions,
        txns.into_iter()
            .skip(first_ver_to_backup as usize)
            .cloned()
            .collect::<Vec<_>>()
    );

    rt.shutdown_timeout(Duration::from_secs(1));
}
