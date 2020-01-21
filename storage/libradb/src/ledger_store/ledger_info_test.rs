// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{change_set::ChangeSet, LibraDB};
use libra_temppath::TempPath;
use libra_types::{
    proptest_types::{AccountInfoUniverse, LedgerInfoWithSignaturesGen},
    transaction::Version,
};
use proptest::{collection::vec, prelude::*};
use std::path::Path;

fn arb_ledger_infos_with_sigs() -> impl Strategy<Value = Vec<LedgerInfoWithSignatures>> {
    (
        any_with::<AccountInfoUniverse>(3),
        vec((any::<LedgerInfoWithSignaturesGen>(), 1..10usize), 1..100),
    )
        .prop_map(|(mut universe, gens)| {
            let ledger_infos_with_sigs: Vec<_> = gens
                .into_iter()
                .map(|(ledger_info_gen, block_size)| {
                    ledger_info_gen.materialize(&mut universe, block_size)
                })
                .collect();
            assert_eq!(get_first_epoch(&ledger_infos_with_sigs), 0);
            ledger_infos_with_sigs
        })
}

fn get_first_epoch(ledger_infos_with_sigs: &[LedgerInfoWithSignatures]) -> u64 {
    ledger_infos_with_sigs
        .first()
        .unwrap()
        .ledger_info()
        .epoch()
}

fn get_last_epoch(ledger_infos_with_sigs: &[LedgerInfoWithSignatures]) -> u64 {
    ledger_infos_with_sigs.last().unwrap().ledger_info().epoch()
}

fn get_last_version(ledger_infos_with_sigs: &[LedgerInfoWithSignatures]) -> Version {
    ledger_infos_with_sigs
        .last()
        .unwrap()
        .ledger_info()
        .version()
}

fn get_num_epoch_changes(ledger_infos_with_sigs: &[LedgerInfoWithSignatures]) -> usize {
    ledger_infos_with_sigs
        .iter()
        .filter(|x| x.ledger_info().next_validator_set().is_some())
        .count()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_get_first_n_epoch_change_ledger_infos(
        (ledger_infos_with_sigs, start_epoch, end_epoch, limit) in arb_ledger_infos_with_sigs()
            .prop_flat_map(|ledger_infos_with_sigs| {
                let first_epoch = get_first_epoch(&ledger_infos_with_sigs);
                let last_epoch = get_last_epoch(&ledger_infos_with_sigs);
                (
                    Just(ledger_infos_with_sigs),
                    first_epoch..=last_epoch,
                )
            })
            .prop_flat_map(|(ledger_infos_with_sigs, start_epoch)| {
                let last_epoch = get_last_epoch(&ledger_infos_with_sigs);
                let num_epoch_changes = get_num_epoch_changes(&ledger_infos_with_sigs);
                assert!(num_epoch_changes >= 1);
                (
                    Just(ledger_infos_with_sigs),
                    Just(start_epoch),
                    (start_epoch..=last_epoch),
                    1..num_epoch_changes * 2,
                )
            })
    ) {
        let tmp_dir = TempPath::new();
        let db = set_up(&tmp_dir, &ledger_infos_with_sigs);

        let (actual, more) = db
            .ledger_store
            .get_first_n_epoch_change_ledger_infos(start_epoch, end_epoch, limit)
            .unwrap();
        let all_epoch_changes = ledger_infos_with_sigs
            .into_iter()
            .filter(|ledger_info_with_sigs| {
                let li = ledger_info_with_sigs.ledger_info();
                start_epoch <= li.epoch()
                    && li.epoch() < end_epoch
                    && li.next_validator_set().is_some()
            })
            .collect::<Vec<_>>();
        prop_assert_eq!(more, all_epoch_changes.len() > limit);

        let expected: Vec<_> = all_epoch_changes.into_iter().take(limit).collect();
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_epoch(
        (ledger_infos_with_sigs, version) in arb_ledger_infos_with_sigs()
            .prop_flat_map(|ledger_infos_with_sigs| {
                let last_version = get_last_version(&ledger_infos_with_sigs);
                (
                    Just(ledger_infos_with_sigs),
                    0..=last_version,
                )
            })
    ) {
        let tmp_dir = TempPath::new();
        let db = set_up(&tmp_dir, &ledger_infos_with_sigs);

        let actual = db.ledger_store.get_epoch(version).unwrap();
        // Find the first LI that is at or after version.
        let index = ledger_infos_with_sigs
            .iter()
            .position(|x| x.ledger_info().version() >= version)
            .unwrap();
        let expected = ledger_infos_with_sigs[index].ledger_info().epoch();
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_startup_info(ledger_infos_with_sigs in arb_ledger_infos_with_sigs()) {
        let tmp_dir = TempPath::new();
        let db = set_up(&tmp_dir, &ledger_infos_with_sigs);

        let (actual_latest_li, actual_vs_opt) = db.ledger_store.get_startup_info().unwrap().unwrap();

        let expected_latest_li = ledger_infos_with_sigs.last().unwrap();
        prop_assert_eq!(&actual_latest_li, expected_latest_li);

        if expected_latest_li.ledger_info().next_validator_set().is_some() {
            prop_assert_eq!(actual_vs_opt, None);
        } else {
            let expected_vs_opt = ledger_infos_with_sigs
                .iter()
                .rev()
                .filter_map(|x| x.ledger_info().next_validator_set().cloned())
                .next()
                .unwrap();
            prop_assert_eq!(actual_vs_opt.unwrap(), expected_vs_opt);
        }
    }
}

fn set_up(path: &impl AsRef<Path>, ledger_infos_with_sigs: &[LedgerInfoWithSignatures]) -> LibraDB {
    let db = LibraDB::new(path);
    let store = &db.ledger_store;

    // Write LIs to DB.
    let mut cs = ChangeSet::new();
    ledger_infos_with_sigs
        .iter()
        .map(|info| store.put_ledger_info(info, &mut cs))
        .collect::<Result<Vec<_>>>()
        .unwrap();
    store.db.write_schemas(cs.batch).unwrap();
    store.set_latest_ledger_info(ledger_infos_with_sigs.last().unwrap().clone());

    db
}
