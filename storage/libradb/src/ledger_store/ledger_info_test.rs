// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{change_set::ChangeSet, LibraDB};
use libra_tools::tempdir::TempPath;
use libra_types::proptest_types::{AccountInfoUniverse, LedgerInfoWithSignaturesGen};
use proptest::{collection::vec, prelude::*};

fn arb_ledger_infos_with_sigs() -> impl Strategy<Value = Vec<LedgerInfoWithSignatures>> {
    (
        any_with::<AccountInfoUniverse>(3),
        vec((any::<LedgerInfoWithSignaturesGen>(), 1..10usize), 1..100),
    )
        .prop_map(|(mut universe, gens)| {
            gens.into_iter()
                .map(|(ledger_info_gen, block_size)| {
                    ledger_info_gen.materialize(&mut universe, block_size)
                })
                .collect()
        })
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

        // verify get_epoch_change_ledger_infos()
        let epoch_ledgers: Vec<_> = ledger_infos_with_sigs
            .iter()
            .filter(|info| info.ledger_info().next_validator_set().is_some())
            .cloned()
            .collect();
        let epoch_change_versions: Vec<_> = epoch_ledgers.iter().map(|li| {
            let ledger_info = li.ledger_info();
            (ledger_info.epoch(), ledger_info.version())
        }).collect();
        let start_epoch = ledger_infos_with_sigs.first().unwrap().ledger_info().epoch();
        let latest_ledger_info = ledger_infos_with_sigs.last().unwrap().ledger_info();
        let latest_epoch = latest_ledger_info.epoch();
        let proof = store.get_epoch_change_ledger_infos(start_epoch, latest_epoch).unwrap();
        let expected_proof = epoch_ledgers
                .into_iter()
                .filter(|li| li.ledger_info().epoch() < latest_epoch)
                .collect::<Vec<_>>();
        prop_assert_eq!(&proof, &expected_proof);

        // verify get_epoch()
        for pair in (&epoch_change_versions).windows(2) {
            let prev = pair[0];
            let prev_epoch = prev.0;
            let prev_epoch_last_ver = prev.1;
            let this = pair[1];
            let this_epoch = this.0;
            let this_epoch_last_ver = this.1;
            prop_assert_eq!(prev_epoch + 1, this_epoch);

            let mid_ver = (prev_epoch_last_ver + this_epoch_last_ver) / 2;
            let expected_for_mid_ver = if mid_ver > prev_epoch_last_ver { this_epoch } else { prev_epoch };
            prop_assert_eq!
            (
                store.get_epoch(mid_ver).unwrap(), expected_for_mid_ver,
                "prev_epoch_last_ver: {}, this_epoch_last_ver: {}, mid_ver: {}",
                    prev_epoch_last_ver, this_epoch_last_ver, mid_ver
            );
            prop_assert_eq!(store.get_epoch(this_epoch_last_ver).unwrap(), this_epoch, "{}", this_epoch_last_ver);
        }
    }
}
