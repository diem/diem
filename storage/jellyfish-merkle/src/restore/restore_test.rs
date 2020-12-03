// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    mock_tree_store::MockTreeStore, restore::JellyfishMerkleRestore, test_helper::init_mock_db,
    JellyfishMerkleTree, TreeReader,
};
use diem_crypto::HashValue;
use diem_types::{account_state_blob::AccountStateBlob, transaction::Version};
use proptest::{collection::btree_map, prelude::*};
use std::{collections::BTreeMap, sync::Arc};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_restore_without_interruption(
        btree in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 1..1000),
        target_version in 0u64..2000,
    ) {
        let restore_db = Arc::new(MockTreeStore::default());
        // For this test, restore everything without interruption.
        restore_without_interruption(&btree, target_version, &restore_db, true);
    }

    #[test]
    fn test_restore_with_interruption(
        (all, batch1_size) in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 2..1000)
            .prop_flat_map(|btree| {
                let len = btree.len();
                (Just(btree), 1..len)
            })
    ) {
        let (db, version) = init_mock_db(&all.clone().into_iter().collect());
        let tree = JellyfishMerkleTree::new(&db);
        let expected_root_hash = tree.get_root_hash(version).unwrap();
        let batch1: Vec<_> = all.clone().into_iter().take(batch1_size).collect();

        let restore_db = Arc::new(MockTreeStore::default());
        {
            let mut restore =
                JellyfishMerkleRestore::new(Arc::clone(&restore_db), version, expected_root_hash).unwrap();
            let proof = tree
                .get_range_proof(batch1.last().map(|(key, _value)| *key).unwrap(), version)
                .unwrap();
            restore.add_chunk(batch1, proof).unwrap();
            // Do not call `finish`.
        }

        {
            let rightmost_key = match restore_db.get_rightmost_leaf().unwrap() {
                None => {
                    // Sometimes the batch is too small so nothing is written to DB.
                    return Ok(());
                }
                Some((_, node)) => node.account_key(),
            };
            let remaining_accounts: Vec<_> = all
                .clone()
                .into_iter()
                .filter(|(k, _v)| *k > rightmost_key)
                .collect();

            let mut restore =
                JellyfishMerkleRestore::new(Arc::clone(&restore_db), version, expected_root_hash).unwrap();
            let proof = tree
                .get_range_proof(
                    remaining_accounts.last().map(|(key, _value)| *key).unwrap(),
                    version,
                )
                .unwrap();
            restore.add_chunk(remaining_accounts, proof).unwrap();
            restore.finish().unwrap();
        }

        assert_success(&restore_db, expected_root_hash, &all, version);
    }

    #[test]
    fn test_overwrite(
        btree1 in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 1..1000),
        btree2 in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 1..1000),
        target_version in 0u64..2000,
    ) {
        let restore_db = Arc::new(MockTreeStore::new(true /* allow_overwrite */));
        restore_without_interruption(&btree1, target_version, &restore_db, true);
        // overwrite, an entirely different tree
        restore_without_interruption(&btree2, target_version, &restore_db, false);
    }
}

fn assert_success(
    db: &MockTreeStore,
    expected_root_hash: HashValue,
    btree: &BTreeMap<HashValue, AccountStateBlob>,
    version: Version,
) {
    let tree = JellyfishMerkleTree::new(db);
    for (key, value) in btree {
        assert_eq!(tree.get(*key, version).unwrap(), Some(value.clone()));
    }

    let actual_root_hash = tree.get_root_hash(version).unwrap();
    assert_eq!(actual_root_hash, expected_root_hash);
}

fn restore_without_interruption(
    btree: &BTreeMap<HashValue, AccountStateBlob>,
    target_version: Version,
    target_db: &Arc<MockTreeStore>,
    try_resume: bool,
) {
    let (db, source_version) = init_mock_db(&btree.iter().map(|(k, v)| (*k, v.clone())).collect());
    let tree = JellyfishMerkleTree::new(&db);
    let expected_root_hash = tree.get_root_hash(source_version).unwrap();

    let mut restore = if try_resume {
        JellyfishMerkleRestore::new(Arc::clone(target_db), target_version, expected_root_hash)
            .unwrap()
    } else {
        JellyfishMerkleRestore::new_overwrite(
            Arc::clone(target_db),
            target_version,
            expected_root_hash,
        )
        .unwrap()
    };
    for (key, value) in btree {
        let proof = tree.get_range_proof(*key, source_version).unwrap();
        restore
            .add_chunk(vec![(*key, value.clone())], proof)
            .unwrap();
    }
    restore.finish().unwrap();

    assert_success(&target_db, expected_root_hash, &btree, target_version);
}
