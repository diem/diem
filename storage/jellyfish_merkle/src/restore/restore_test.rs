// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    mock_tree_store::MockTreeStore, restore::JellyfishMerkleRestore, JellyfishMerkleTree,
    TreeReader,
};
use crypto::HashValue;
use proptest::{collection::btree_map, prelude::*};
use std::collections::BTreeMap;
use types::{account_state_blob::AccountStateBlob, transaction::Version};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_restore_without_interruption(
        btree in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 1..1000),
    ) {
        let version = (btree.len() - 1) as Version;
        let expected_root_hash = get_expected_root_hash(&btree);

        // For this test, restore everything without interruption.
        let db = MockTreeStore::default();
        let mut restore = JellyfishMerkleRestore::new(&db, version).unwrap();
        for (key, value) in &btree {
            restore.add_chunk(vec![(*key, value.clone())]).unwrap();
        }
        restore.finish().unwrap();

        assert_success(&db, expected_root_hash, &btree, version);
    }

    #[test]
    fn test_restore_with_interruption(
        (all, batch1_size) in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 2..1000)
            .prop_flat_map(|btree| {
                let len = btree.len();
                (Just(btree), 1..len)
            })
    ) {
        let version = (all.len() - 1) as Version;
        let expected_root_hash = get_expected_root_hash(&all);
        let batch1: Vec<_> = all.clone().into_iter().take(batch1_size).collect();
        let db = MockTreeStore::default();

        {
            let mut restore = JellyfishMerkleRestore::new(&db, version).unwrap();
            restore.add_chunk(batch1).unwrap();
            // Do not call `finish`.
        }

        {
            let rightmost_key = match db.get_rightmost_leaf().unwrap() {
                None => {
                    // Sometimes the batch is too small so nothing is written to DB.
                    return Ok(());
                }
                Some((_, node)) => node.account_key(),
            };
            let remaining_accounts: Vec<_> = all.clone()
                .into_iter()
                .filter(|(k, _v)| *k > rightmost_key)
                .collect();
            let mut restore = JellyfishMerkleRestore::new(&db, version).unwrap();
            restore.add_chunk(remaining_accounts).unwrap();
            restore.finish().unwrap();
        }

        assert_success(&db, expected_root_hash, &all, version);
    }
}

fn get_expected_root_hash(btree: &BTreeMap<HashValue, AccountStateBlob>) -> HashValue {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    for (i, (key, value)) in btree.iter().enumerate() {
        let (_root_hash, batch) = tree
            .put_blob_set(vec![(*key, value.clone())], i as Version)
            .unwrap();
        db.write_tree_update_batch(batch).unwrap();
    }
    tree.get_root_hash((btree.len() - 1) as Version).unwrap()
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
