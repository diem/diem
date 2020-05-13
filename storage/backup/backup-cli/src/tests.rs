// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter::local_storage::LocalStorage,
    backup::{backup_account_state, BackupServiceClient},
    restore::restore_account_state,
};
use backup_service::start_backup_service;
use libra_config::config::NodeConfig;
use libra_proptest_helpers::ValueGenerator;
use libra_temppath::TempPath;
use libra_types::transaction::PRE_GENESIS_VERSION;
use libradb::{test_helper::arb_blocks_to_commit, LibraDB};
use std::sync::Arc;
use storage_client::StorageReadServiceClient;
use storage_interface::{DbReader, DbWriter};
use storage_service::start_storage_service_with_db;

fn tmp_db_empty() -> (TempPath, LibraDB) {
    let tmpdir = TempPath::new();
    let db = LibraDB::new_for_test(&tmpdir);

    (tmpdir, db)
}

fn tmp_db_with_random_content() -> (TempPath, LibraDB) {
    let (tmpdir, db) = tmp_db_empty();
    let mut cur_ver = 0;
    for (txns_to_commit, ledger_info_with_sigs) in
        ValueGenerator::new().generate(arb_blocks_to_commit())
    {
        db.save_transactions(
            &txns_to_commit,
            cur_ver, /* first_version */
            Some(&ledger_info_with_sigs),
        )
        .unwrap();
        cur_ver += txns_to_commit.len() as u64;
    }

    (tmpdir, db)
}

#[test]
fn end_to_end() {
    let (_src_db_dir, db_src) = tmp_db_with_random_content();
    let db_src = Arc::new(db_src);
    let (latest_version, state_root_hash) = db_src.get_latest_state_root().unwrap();
    let tgt_db_dir = TempPath::new();
    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let adaptor = LocalStorage::new(backup_dir.path().to_path_buf());

    let config = NodeConfig::random();
    let _rt = start_storage_service_with_db(&config, Arc::clone(&db_src));
    let mut rt = start_backup_service(config.storage.backup_service_port, db_src);
    let backup_service = BackupServiceClient::new(config.storage.backup_service_port);
    let client = StorageReadServiceClient::new(&config.storage.address);
    let handles = rt
        .block_on(backup_account_state(
            &client,
            &backup_service,
            latest_version,
            &adaptor,
            1024 * 1024,
        ))
        .unwrap();

    restore_account_state(
        PRE_GENESIS_VERSION,
        state_root_hash,
        &tgt_db_dir,
        handles.into_iter().map(Ok),
    );
    let tgt_db = LibraDB::open(
        &tgt_db_dir,
        true, /* readonly */
        None, /* pruner */
    )
    .unwrap();
    assert_eq!(
        tgt_db
            .get_latest_tree_state()
            .unwrap()
            .account_state_root_hash,
        state_root_hash,
    );
}
