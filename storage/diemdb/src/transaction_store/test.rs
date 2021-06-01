// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::DiemDB;
use diem_proptest_helpers::Index;
use diem_temppath::TempPath;
use diem_types::{
    block_metadata::BlockMetadata,
    proptest_types::{AccountInfoUniverse, SignatureCheckedTransactionGen},
    transaction::{SignedTransaction, Transaction},
};
use proptest::{collection::vec, prelude::*};
use std::collections::BTreeMap;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_put_get(
        universe in any_with::<AccountInfoUniverse>(3),
        gens in vec(
            (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
            1..10
        ),
    ) {
        let tmp_dir = TempPath::new();
        let db = DiemDB::new_for_test(&tmp_dir);
        let store = &db.transaction_store;
        let txns = init_store(universe, gens, &store);

        let ledger_version = txns.len() as Version - 1;
        for (ver, txn) in txns.iter().enumerate() {
            prop_assert_eq!(store.get_transaction(ver as Version).unwrap(), txn.clone());
            let user_txn = txn
                .as_signed_user_txn()
                .expect("All should be user transactions here.");
            prop_assert_eq!(
                store
                    .lookup_transaction_by_account(
                        user_txn.sender(),
                        user_txn.sequence_number(),
                        ledger_version
                    )
                    .unwrap(),
                Some(ver as Version)
            );
        }

        prop_assert!(store.get_transaction(ledger_version + 1).is_err());
    }

    #[test]
    fn test_get_transaction_iter(
        universe in any_with::<AccountInfoUniverse>(3),
        gens in vec(
            (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
            1..10
        ),
    ) {
        let tmp_dir = TempPath::new();
        let db = DiemDB::new_for_test(&tmp_dir);
        let store = &db.transaction_store;
        let txns = init_store(universe, gens, &store);

        let total_num_txns = txns.len();

        let actual = store
            .get_transaction_iter(0, total_num_txns)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        prop_assert_eq!(actual, txns.clone());

        let actual = store
            .get_transaction_iter(0, total_num_txns + 1)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        prop_assert_eq!(actual, txns.clone());

        let actual = store
            .get_transaction_iter(0, 0)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        prop_assert!(actual.is_empty());

        if total_num_txns > 0 {
            let actual = store
                .get_transaction_iter(0, total_num_txns - 1)
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap();
            prop_assert_eq!(
                actual,
                txns
                    .into_iter()
                    .take(total_num_txns as usize - 1)
                    .collect::<Vec<_>>()
            );
        }

        prop_assert!(store.get_transaction_iter(10, usize::max_value()).is_err());
    }

    #[test]
    fn test_get_block_metadata(
        txns in vec(
            prop_oneof![
                any::<BlockMetadata>().prop_map(Transaction::BlockMetadata),
                any::<SignedTransaction>().prop_map(Transaction::UserTransaction),
            ],
            1..100,
        )
    ) {
        let tmp_dir = TempPath::new();
        let db = DiemDB::new_for_test(&tmp_dir);
        let store = &db.transaction_store;

        let mut cs = ChangeSet::new();
        for (ver, txn) in txns.iter().enumerate() {
            store
                .put_transaction(ver as Version, &txn, &mut cs)
                .unwrap();
        }
        store.db.write_schemas(cs.batch).unwrap();

        let mut timestamp = 0;
        let mut block_meta_ver = 0;
        let mut seen_any_block = false;
        for (ver, txn) in txns.into_iter().enumerate() {
            if let Transaction::BlockMetadata(b) = txn {
                timestamp = b.into_inner().1;
                block_meta_ver = ver as Version;
                seen_any_block = true;
            }
            let block_meta_opt = store.get_block_metadata(ver as Version).unwrap();
            if seen_any_block {
                let (v, block_meta) = block_meta_opt.unwrap();
                prop_assert_eq!(
                    v,
                    block_meta_ver
                );
                prop_assert_eq!(
                    block_meta.into_inner().1,
                    timestamp
                );
            } else {
                prop_assert!(block_meta_opt.is_none());
            }
        }
    }

    #[test]
    fn test_get_account_transaction_version_iter(
        universe in any_with::<AccountInfoUniverse>(5),
        gens in vec(
            (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
            1..=50,
        ),
        seq_num_offset in 0_u64..=10,
        ledger_version in 0_u64..50,
        num_versions in 0_u64..=50,
    ) {
        let tmp_dir = TempPath::new();
        let db = DiemDB::new_for_test(&tmp_dir);
        let store = &db.transaction_store;
        let txns = init_store(universe, gens, &store);

        let txns = txns
            .iter()
            .enumerate()
            .map(|(version, txn)| (version as u64, txn.as_signed_user_txn().unwrap()))
            .collect::<Vec<_>>();

        // can we just get all the account transaction versions individually

        for (version, txn) in &txns {
            let mut iter = store.get_account_transaction_version_iter(
                txn.sender(),
                txn.sequence_number(),
                1, /* num_versions */
                ledger_version,
            ).unwrap();

            if *version <= ledger_version {
                let (actual_seq_num, actual_version) = iter.next().unwrap().unwrap();
                prop_assert!(iter.next().is_none());

                prop_assert_eq!(*version, actual_version);
                prop_assert_eq!(txn.sequence_number(), actual_seq_num);
            } else {
                prop_assert!(iter.next().is_none());
            }
        }

        // now do a full scan of each account

        // what does the expected view look like
        let mut expected_scan = BTreeMap::<AccountAddress, Vec<(u64, Version)>>::new();
        for (version, txn) in &txns {
            let seq_num = txn.sequence_number();
            if *version <= ledger_version && seq_num >= seq_num_offset {
                let txn_metadatas = expected_scan.entry(txn.sender()).or_default();
                if (txn_metadatas.len() as u64) < num_versions {
                    txn_metadatas.push((seq_num, *version));
                }
            }
        }

        // throw in some non-existent accounts; make sure we don't return anything for them
        expected_scan.entry(AccountAddress::from_hex_literal("0x1234").unwrap()).or_default();
        expected_scan.entry(AccountAddress::from_hex_literal("0x77777777").unwrap()).or_default();
        expected_scan.entry(AccountAddress::from_hex_literal("0x42").unwrap()).or_default();

        // scan the db
        let actual_scan = expected_scan
            .keys()
            .map(|address| {
                let txn_metadatas = store
                    .get_account_transaction_version_iter(
                        *address,
                        seq_num_offset,
                        num_versions,
                        ledger_version,
                    )
                    .unwrap()
                    .collect::<Result<Vec<_>>>()
                    .unwrap();
                (*address, txn_metadatas)
            })
            .collect::<BTreeMap<_, _>>();

        prop_assert_eq!(&actual_scan, &expected_scan);
    }
}

fn init_store(
    mut universe: AccountInfoUniverse,
    gens: Vec<(Index, SignatureCheckedTransactionGen)>,
    store: &TransactionStore,
) -> Vec<Transaction> {
    let txns = gens
        .into_iter()
        .map(|(index, gen)| {
            Transaction::UserTransaction(gen.materialize(*index, &mut universe).into_inner())
        })
        .collect::<Vec<_>>();

    assert!(store.get_transaction(0).is_err());

    let mut cs = ChangeSet::new();
    for (ver, txn) in txns.iter().enumerate() {
        store
            .put_transaction(ver as Version, &txn, &mut cs)
            .unwrap();
    }
    store.db.write_schemas(cs.batch).unwrap();

    txns
}
