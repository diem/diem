// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::LibraDB;
use libra_proptest_helpers::Index;
use libra_temppath::TempPath;
use libra_types::proptest_types::{AccountInfoUniverse, SignatureCheckedTransactionGen};
use proptest::{collection::vec, prelude::*};

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
        let db = LibraDB::new(&tmp_dir);
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
        let db = LibraDB::new(&tmp_dir);
        let store = &db.transaction_store;
        let txns = init_store(universe, gens, &store);

        let total_num_txns = txns.len() as u64;

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

        prop_assert!(store.get_transaction_iter(10, u64::max_value()).is_err());
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
            Transaction::UserTransaction(gen.materialize(index, &mut universe).into_inner())
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
