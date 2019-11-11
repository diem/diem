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
        vec((any::<LedgerInfoWithSignaturesGen>(), 0..10usize), 1..100),
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
        let start_epoch = ledger_infos_with_sigs.first().unwrap().ledger_info().epoch();
        let ledger_version = ledger_infos_with_sigs.last().unwrap().ledger_info().version();
        prop_assert_eq!(
            &store.get_epoch_change_ledger_infos(start_epoch, ledger_version).unwrap(),
            &epoch_ledgers
        );

        // verify get_epoch()
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
            prop_assert_eq!(prev_epoch + 1, this_epoch);

            let mid_ver = (prev_ver + this_ver) / 2;
            prop_assert_eq!(store.get_epoch(mid_ver).unwrap(), prev_epoch, "{}", mid_ver);
            prop_assert_eq!(store.get_epoch(this_ver).unwrap(), this_epoch, "{}", this_ver);
        }
    }
}
