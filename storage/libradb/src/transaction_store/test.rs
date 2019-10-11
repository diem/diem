// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::LibraDB;
use libra_types::proptest_types::{AccountInfoUniverse, SignatureCheckedTransactionGen};
use proptest::{collection::vec, prelude::*};
use proptest_helpers::Index;
use tools::tempdir::TempPath;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_put_get(
        mut universe in any_with::<AccountInfoUniverse>(3),
        gens in vec(
            (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
            1..10
        ),
    ) {
        let txns = gens
            .into_iter()
            .map(|(index, gen)| gen.materialize(index, &mut universe).into_inner())
            .collect::<Vec<_>>();

        let tmp_dir = TempPath::new();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.transaction_store;

        prop_assert!(store.get_transaction(0).is_err());

        let mut cs = ChangeSet::new();
        for (ver, txn) in txns.iter().enumerate() {
            store
                .put_transaction(ver as Version, &Transaction::UserTransaction(txn.clone()), &mut cs)
                .unwrap();
        }
        store.db.write_schemas(cs.batch).unwrap();

        let ledger_version = txns.len() as Version - 1;
        for (ver, txn) in txns.iter().enumerate() {
            prop_assert_eq!(store.get_transaction(ver as Version).unwrap(), txn.clone());
            prop_assert_eq!(
                store
                    .lookup_transaction_by_account(
                        txn.sender(),
                        txn.sequence_number(),
                        ledger_version
                    )
                    .unwrap(),
                Some(ver as Version)
            );
        }

        prop_assert!(store.get_transaction(ledger_version + 1).is_err());
    }
}
