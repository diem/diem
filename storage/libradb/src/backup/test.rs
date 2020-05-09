// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_helper::arb_blocks_to_commit, LibraDB};
use anyhow::Result;
use libra_temppath::TempPath;
use proptest::prelude::*;
use storage_interface::DbWriter;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_get_transaction_iter(input in arb_blocks_to_commit()) {
        let tmp_dir = TempPath::new();
        let db = LibraDB::new_for_test(&tmp_dir);

        let mut cur_ver = 0;
        for (txns_to_commit, ledger_info_with_sigs) in input.iter() {
            db.save_transactions(&txns_to_commit, cur_ver, Some(ledger_info_with_sigs))
                .unwrap();
            cur_ver += txns_to_commit.len() as u64;
        }

        let expected: Vec<_> = input
            .iter()
            .flat_map(|(txns_to_commit, _ledger_info_with_sigs)| {
                txns_to_commit
                    .iter()
                    .map(|txn_to_commit| txn_to_commit.transaction().clone())
            })
            .collect();
        prop_assert_eq!(expected.len() as u64, cur_ver);

        let actual = db
            .get_backup_handler()
            .get_transaction_iter(0, cur_ver)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        prop_assert_eq!(actual, expected);
    }
}
