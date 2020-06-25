// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_proptest_helpers::ValueGenerator;
use libra_temppath::TempPath;
use libra_types::{ledger_info::LedgerInfoWithSignatures, transaction::TransactionToCommit};
use libradb::{test_helper::arb_blocks_to_commit, LibraDB};
use std::sync::Arc;
use storage_interface::DbWriter;

pub fn tmp_db_empty() -> (TempPath, Arc<LibraDB>) {
    let tmpdir = TempPath::new();
    let db = Arc::new(LibraDB::new_for_test(&tmpdir));

    (tmpdir, db)
}

pub fn tmp_db_with_random_content() -> (
    TempPath,
    Arc<LibraDB>,
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
