// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::LibraDB;
use proptest::{collection::vec, prelude::*};
use tempfile::tempdir;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_put_get(txns in vec(any::<SignedTransaction>(), 1..10)) {
        let tmp_dir = tempdir().unwrap();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.transaction_store;

        prop_assert!(store.get_transaction(0).is_err());

        let mut batch = SchemaBatch::new();
        for (i, txn) in txns.iter().enumerate() {
            store.put_transaction(i as u64, &txn, &mut batch).unwrap();
        }
        db.commit(batch).unwrap();

        for (i, txn) in txns.iter().enumerate() {
            prop_assert_eq!(store.get_transaction(i as u64).unwrap(), txn.clone());
        }

        prop_assert!(store.get_transaction(txns.len() as u64).is_err());
    }
}
