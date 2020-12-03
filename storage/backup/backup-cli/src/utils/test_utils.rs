// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_service::start_backup_service;
use diem_config::utils::get_available_port;
use diem_proptest_helpers::ValueGenerator;
use diem_temppath::TempPath;
use diem_types::{ledger_info::LedgerInfoWithSignatures, transaction::TransactionToCommit};
use diemdb::{test_helper::arb_blocks_to_commit, DiemDB};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use storage_interface::DbWriter;
use tokio::runtime::Runtime;

pub fn tmp_db_empty() -> (TempPath, Arc<DiemDB>) {
    let tmpdir = TempPath::new();
    let db = Arc::new(DiemDB::new_for_test(&tmpdir));

    (tmpdir, db)
}

pub fn tmp_db_with_random_content() -> (
    TempPath,
    Arc<DiemDB>,
    Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) {
    let (tmpdir, db) = tmp_db_empty();
    let mut cur_ver = 0;
    let blocks = ValueGenerator::new().generate(arb_blocks_to_commit());
    for (txns_to_commit, ledger_info_with_sigs) in &blocks {
        db.save_transactions(
            txns_to_commit,
            cur_ver, /* first_version */
            Some(ledger_info_with_sigs),
        )
        .unwrap();
        cur_ver += txns_to_commit.len() as u64;
    }

    (tmpdir, db, blocks)
}

pub fn start_local_backup_service(db: Arc<DiemDB>) -> (Runtime, u16) {
    let port = get_available_port();
    let rt = start_backup_service(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port), db);
    (rt, port)
}
