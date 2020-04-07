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
        vec((any::<LedgerInfoWithSignaturesGen>(), 1..10usize), 1..10),
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
    fn test_get_validator_set(ledger_infos_with_sigs in arb_ledger_infos_with_sigs()) {
        let tmp_dir = TempPath::new();
        let db = set_up(&tmp_dir, &ledger_infos_with_sigs);

        assert!(db.ledger_store.get_validator_set(0).is_err());

        for li_with_sigs in ledger_infos_with_sigs {
            let li = li_with_sigs.ledger_info();
            if li.next_validator_set().is_some() {
                assert_eq!(
                    db.ledger_store.get_validator_set(li.epoch()+1).unwrap(),
                    *li.next_validator_set().unwrap(),
                );
            }

        }
    }

    #[test]
    fn test_get_startup_info(
        (ledger_infos_with_sigs, txn_infos) in arb_ledger_infos_with_sigs()
            .prop_flat_map(|lis| {
                let num_committed_txns = get_last_version(&lis) as usize + 1;
                (
                    Just(lis),
                    vec(any::<TransactionInfo>(), num_committed_txns..num_committed_txns + 10),
                )
            })
    ) {
        let tmp_dir = TempPath::new();
        let db = set_up(&tmp_dir, &ledger_infos_with_sigs);
        put_transaction_infos(&db, &txn_infos);

        let startup_info = db.ledger_store.get_startup_info().unwrap().unwrap();
        let latest_li = ledger_infos_with_sigs.last().unwrap().ledger_info();
        assert_eq!(startup_info.latest_ledger_info, *ledger_infos_with_sigs.last().unwrap());
        let expected_validator_set = if latest_li.next_validator_set().is_none() {
            Some(db.ledger_store.get_validator_set(latest_li.epoch()).unwrap())
        } else {
            None
        };
        assert_eq!(startup_info.latest_validator_set, expected_validator_set);
        let committed_version = get_last_version(&ledger_infos_with_sigs);
        assert_eq!(
            startup_info.committed_tree_state.account_state_root_hash,
            txn_infos[committed_version as usize].state_root_hash(),
        );
        let synced_version = (txn_infos.len() - 1) as u64;
        if synced_version > committed_version {
            assert_eq!(
                startup_info.synced_tree_state.unwrap().account_state_root_hash,
                txn_infos.last().unwrap().state_root_hash(),
            );
        } else {
            assert!(startup_info.synced_tree_state.is_none());
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

fn put_transaction_infos(db: &LibraDB, txn_infos: &[TransactionInfo]) {
    let mut cs = ChangeSet::new();
    db.ledger_store
        .put_transaction_infos(0, txn_infos, &mut cs)
        .unwrap();
    db.db.write_schemas(cs.batch).unwrap()
}
