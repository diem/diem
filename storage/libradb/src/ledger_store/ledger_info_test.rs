// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{change_set::ChangeSet, LibraDB};
use libra_tools::tempdir::TempPath;
use libra_types::block_info::BlockInfo;
use libra_types::ledger_info::LedgerInfo;
use libra_types::validator_set::ValidatorSet;
use proptest::{collection::vec, prelude::*};

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
                let version = start_version + i as u64;
                let epoch: u64 = version / 3;
                // epoch changes at versions that are multiplications of 3
                let next_validator_set = if version % 3 == 2 {
                    Some(ValidatorSet::new(vec![]))
                } else {
                    None
                };

                LedgerInfoWithSignatures::new(
                    LedgerInfo::new(
                        BlockInfo::new(
                            version,
                            epoch,
                            ledger_info.consensus_block_id(),
                            ledger_info.transaction_accumulator_hash(),
                            start_epoch + i as Version,
                            ledger_info.timestamp_usecs(),
                            next_validator_set,
                        ),
                        ledger_info.consensus_data_hash(),
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
        // set up
        let tmp_dir = TempPath::new();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.ledger_store;

        // write to DB
        let mut cs = ChangeSet::new();
        ledger_infos_with_sigs
            .iter()
            .map(|info| store.put_ledger_info(info, &mut cs))
            .collect::<Result<Vec<_>>>()
            .unwrap();
        store.db.write_schemas(cs.batch).unwrap();

        // verify get_latest_ledger_infos_per_epoch()
        let mut epoch_ledgers: Vec<_> = ledger_infos_with_sigs
            .iter()
            .filter(|x| x.ledger_info().version() % 3 == 2)
            .map(|x| x.clone())
            .collect();
        let last = ledger_infos_with_sigs.last().unwrap();
        if last.ledger_info().version() % 3 != 2 {
            epoch_ledgers.push(last.clone());
        }
        let start_epoch = ledger_infos_with_sigs.first().unwrap().ledger_info().epoch();
        prop_assert_eq!(
            db.ledger_store.get_latest_ledger_infos_per_epoch(start_epoch).unwrap(),
            epoch_ledgers
        );

        // verify get_epoch()
        let mut seen_first_epoch_change = false;
        for x in ledger_infos_with_sigs {
            if seen_first_epoch_change {
                prop_assert_eq!(
                    db.ledger_store.get_epoch(x.ledger_info().version()).unwrap(),
                    x.ledger_info().epoch()
                );
            } else if x.ledger_info().version() % 3 == 2 {
                seen_first_epoch_change = true;
            }
        }

    }
}
