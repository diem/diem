// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::LibraDB;
use proptest::{collection::vec, prelude::*};
use tempfile::tempdir;
use types::ledger_info::LedgerInfo;

prop_compose! {
    fn arb_partial_ledger_info()(accu_hash in any::<HashValue>(),
                                 consensus_hash in any::<HashValue>(),
                                 timestamp in any::<u64>()) -> (HashValue, HashValue, u64) {
        (accu_hash, consensus_hash, timestamp)
    }
}

prop_compose! {
    fn arb_ledger_infos_with_sigs()(
        partial_ledger_infos_with_sigs in vec(
            any_with::<LedgerInfoWithSignatures>((1..3).into()).no_shrink(), 1..100
        ),
        start_version in 0..10000u64,
    ) -> Vec<LedgerInfoWithSignatures> {
        partial_ledger_infos_with_sigs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let ledger_info = p.ledger_info();
                LedgerInfoWithSignatures::new(
                    LedgerInfo::new(
                        start_version + i as u64,
                        ledger_info.transaction_accumulator_hash(),
                        ledger_info.consensus_data_hash(),
                        HashValue::zero(),
                        ledger_info.epoch_num(),
                        ledger_info.timestamp_usecs(),
                    ),
                    p.signatures().clone(),
                )
            })
            .collect()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_ledger_info_put_get_verify(
        ledger_infos_with_sigs in arb_ledger_infos_with_sigs()
    ) {
        let tmp_dir = tempdir().unwrap();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.ledger_store;
        let start_version = ledger_infos_with_sigs.first().unwrap().ledger_info().version();

        let mut batch = SchemaBatch::new();
        ledger_infos_with_sigs
            .iter()
            .map(|info| store.put_ledger_info(info, &mut batch))
            .collect::<Result<Vec<_>>>()
            .unwrap();
        db.commit(batch).unwrap();
        prop_assert_eq!(db.ledger_store.get_ledger_infos(start_version).unwrap(), ledger_infos_with_sigs);
    }
}
