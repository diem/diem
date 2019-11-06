// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{change_set::ChangeSet, LibraDB};
use libra_tools::tempdir::TempPath;
use libra_types::block_info::BlockInfo;
use libra_types::ledger_info::LedgerInfo;
use libra_types::validator_set::ValidatorSet;
use proptest::{collection::vec, prelude::*};
use std::collections::BTreeMap;

fn arb_ledger_infos_with_sigs() -> impl Strategy<Value = Vec<(LedgerInfoWithSignatures, bool)>> {
    (
        vec(
            (
                any::<HashValue>(),
                any::<HashValue>(),
                any::<HashValue>(),
                any::<u64>(),
                0..10u64,
                any::<bool>(),
            ),
            1..100,
        ),
        vec(0..100_000_000u64, 3),
    )
        .prop_map(|(block_params, mut ledger_params)| {
            ledger_params.sort_unstable();
            let mut epoch = ledger_params[0];
            let mut round = ledger_params[1];
            let mut version = ledger_params[2];

            block_params
                .into_iter()
                .map(
                    |(
                        block_id,
                        accu_hash,
                        consensus_data_hash,
                        timestamp,
                        block_size,
                        new_epoch_if_possible,
                    )| {
                        round += 1;
                        version += block_size;
                        let is_new_epoch = new_epoch_if_possible && block_size != 0;
                        let next_validator_set = if is_new_epoch {
                            Some(ValidatorSet::new(Vec::new()))
                        } else {
                            None
                        };

                        let li = LedgerInfoWithSignatures::new(
                            LedgerInfo::new(
                                BlockInfo::new(
                                    epoch,
                                    round,
                                    block_id,
                                    accu_hash,
                                    version,
                                    timestamp,
                                    next_validator_set,
                                ),
                                consensus_data_hash,
                            ),
                            BTreeMap::default(),
                        );
                        if is_new_epoch {
                            epoch += 1;
                        }
                        (li, is_new_epoch)
                    },
                )
                .collect()
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_ledger_info_put_get_verify(
        inputs in arb_ledger_infos_with_sigs()
    ) {
        // set up
        let tmp_dir = TempPath::new();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.ledger_store;

        // write to DB
        let mut cs = ChangeSet::new();
        inputs
            .iter()
            .map(|(info, _)| store.put_ledger_info(info, &mut cs))
            .collect::<Result<Vec<_>>>()
            .unwrap();
        store.db.write_schemas(cs.batch).unwrap();

        // verify get_latest_ledger_infos_per_epoch()
        let mut epoch_ledgers: Vec<_> = inputs
            .iter()
            .filter(|(_, is_new_epoch)| *is_new_epoch)
            .map(|(li, _)| li.clone())
            .collect();
        let last = inputs.last().unwrap();
        let last_is_new_epoch = last.1;
        if !last_is_new_epoch {
            epoch_ledgers.push(last.0.clone());
        }
        let start_epoch = inputs.first().unwrap().0.ledger_info().epoch();
        prop_assert_eq!(
            &store.get_latest_ledger_infos_per_epoch(start_epoch).unwrap(),
            &epoch_ledgers
        );

        // verify get_epoch()
        if !last_is_new_epoch {
            epoch_ledgers.pop();
        }
        let epoch_change_versions: Vec<_> = epoch_ledgers.iter().map(|li| {
            let ledger_info = li.ledger_info();
            (ledger_info.epoch() + 1, ledger_info.version())
        }).collect();
        for pair in (&epoch_change_versions).windows(2) {
            let prev = pair[0];
            let prev_epoch = prev.0;
            let prev_ver = prev.1;
            let this = pair[1];
            let this_epoch = this.0;
            let this_ver = this.1;

            let mid_ver = (prev_ver + this_ver) / 2;
            prop_assert_eq!(store.get_epoch(mid_ver).unwrap(), prev_epoch, "{}", mid_ver);
            prop_assert_eq!(store.get_epoch(this_ver).unwrap(), this_epoch, "{}", this_ver);
        }
    }
}
