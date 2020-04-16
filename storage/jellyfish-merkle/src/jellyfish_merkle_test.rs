// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use libra_nibble::Nibble;
use libra_types::{proof::SparseMerkleInternalNode, transaction::PRE_GENESIS_VERSION};
use mock_tree_store::MockTreeStore;
use proptest::{
    collection::{btree_map, hash_map, vec},
    prelude::*,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{collections::HashMap, ops::Bound};
use test_helper::{init_mock_db, plus_one};

fn update_nibble(original_key: &HashValue, n: usize, nibble: u8) -> HashValue {
    assert!(nibble < 16);
    let mut key = original_key.to_vec();
    key[n / 2] = if n % 2 == 0 {
        key[n / 2] & 0x0f | nibble << 4
    } else {
        key[n / 2] & 0xf0 | nibble
    };
    HashValue::from_slice(&key).unwrap()
}

#[test]
fn test_insert_to_empty_tree() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    // Tree is initially empty. Root is a null node. We'll insert a key-value pair which creates a
    // leaf node.
    let key = HashValue::random();
    let value = AccountStateBlob::from(vec![1u8, 2u8, 3u8, 4u8]);

    let (_new_root_hash, batch) = tree
        .put_blob_set(vec![(key, value.clone())], 0 /* version */)
        .unwrap();
    assert!(batch.stale_node_index_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();

    assert_eq!(tree.get(key, 0).unwrap().unwrap(), value);
}

#[test]
fn test_insert_to_pre_genesis() {
    // Set up DB with pre-genesis state (one single leaf node).
    let db = MockTreeStore::default();
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);
    let pre_genesis_root_key = NodeKey::new_empty_path(PRE_GENESIS_VERSION);
    db.put_node(pre_genesis_root_key, Node::new_leaf(key1, value1.clone()))
        .unwrap();

    // Genesis inserts one more leaf.
    let tree = JellyfishMerkleTree::new(&db);
    let key2 = update_nibble(&key1, 0, 15);
    let value2 = AccountStateBlob::from(vec![3u8, 4u8]);
    let (_root_hash, batch) = tree
        .put_blob_set(vec![(key2, value2.clone())], 0 /* version */)
        .unwrap();

    // Check pre-genesis node prunes okay.
    assert_eq!(batch.stale_node_index_batch.len(), 1);
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(db.num_nodes(), 4);
    db.purge_stale_nodes(0).unwrap();
    assert_eq!(db.num_nodes(), 3);

    // Check mixed state reads okay.
    assert_eq!(tree.get(key1, 0).unwrap().unwrap(), value1);
    assert_eq!(tree.get(key2, 0).unwrap().unwrap(), value2);
}

#[test]
fn test_insert_at_leaf_with_internal_created() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (_root0_hash, batch) = tree
        .put_blob_set(vec![(key1, value1.clone())], 0 /* version */)
        .unwrap();

    assert!(batch.stale_node_index_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, 0).unwrap().unwrap(), value1);

    // Insert at the previous leaf node. Should generate an internal node at the root.
    // Change the 1st nibble to 15.
    let key2 = update_nibble(&key1, 0, 15);
    let value2 = AccountStateBlob::from(vec![3u8, 4u8]);

    let (_root1_hash, batch) = tree
        .put_blob_set(vec![(key2, value2.clone())], 1 /* version */)
        .unwrap();
    assert_eq!(batch.stale_node_index_batch.len(), 1);
    db.write_tree_update_batch(batch).unwrap();

    assert_eq!(tree.get(key1, 0).unwrap().unwrap(), value1);
    assert!(tree.get(key2, 0).unwrap().is_none());
    assert_eq!(tree.get(key2, 1).unwrap().unwrap(), value2);

    // get # of nodes
    assert_eq!(db.num_nodes(), 4 /* 1 + 3 */);

    let internal_node_key = NodeKey::new_empty_path(1);

    let leaf1 = Node::new_leaf(key1, value1);
    let leaf2 = Node::new_leaf(key2, value2);
    let mut children = HashMap::new();
    children.insert(
        Nibble::from(0),
        Child::new(leaf1.hash(), 1 /* version */, true /* is_leaf */),
    );
    children.insert(
        Nibble::from(15),
        Child::new(leaf2.hash(), 1 /* version */, true /* is_leaf */),
    );
    let internal = Node::new_internal(children);
    assert_eq!(db.get_node(&NodeKey::new_empty_path(0)).unwrap(), leaf1);
    assert_eq!(
        db.get_node(&internal_node_key.gen_child_node_key(1 /* version */, Nibble::from(0)))
            .unwrap(),
        leaf1
    );
    assert_eq!(
        db.get_node(&internal_node_key.gen_child_node_key(1 /* version */, Nibble::from(15)))
            .unwrap(),
        leaf2
    );
    assert_eq!(db.get_node(&internal_node_key).unwrap(), internal);
}

#[test]
fn test_insert_at_leaf_with_multiple_internals_created() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    // 1. Insert the first leaf into empty tree
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (_root0_hash, batch) = tree
        .put_blob_set(vec![(key1, value1.clone())], 0 /* version */)
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, 0).unwrap().unwrap(), value1);

    // 2. Insert at the previous leaf node. Should generate a branch node at root.
    // Change the 2nd nibble to 1.
    let key2 = update_nibble(&key1, 1 /* nibble_index */, 1 /* nibble */);
    let value2 = AccountStateBlob::from(vec![3u8, 4u8]);

    let (_root1_hash, batch) = tree
        .put_blob_set(vec![(key2, value2.clone())], 1 /* version */)
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, 0).unwrap().unwrap(), value1);
    assert!(tree.get(key2, 0).unwrap().is_none());
    assert_eq!(tree.get(key2, 1).unwrap().unwrap(), value2);

    assert_eq!(db.num_nodes(), 5);

    let internal_node_key = NodeKey::new(1, NibblePath::new_odd(vec![0x00]));

    let leaf1 = Node::new_leaf(key1, value1.clone());
    let leaf2 = Node::new_leaf(key2, value2.clone());
    let internal = {
        let mut children = HashMap::new();
        children.insert(
            Nibble::from(0),
            Child::new(leaf1.hash(), 1 /* version */, true /* is_leaf */),
        );
        children.insert(
            Nibble::from(1),
            Child::new(leaf2.hash(), 1 /* version */, true /* is_leaf */),
        );
        Node::new_internal(children)
    };

    let root_internal = {
        let mut children = HashMap::new();
        children.insert(
            Nibble::from(0),
            Child::new(
                internal.hash(),
                1,     /* version */
                false, /* is_leaf */
            ),
        );
        Node::new_internal(children)
    };

    assert_eq!(db.get_node(&NodeKey::new_empty_path(0)).unwrap(), leaf1);
    assert_eq!(
        db.get_node(&internal_node_key.gen_child_node_key(1 /* version */, Nibble::from(0)))
            .unwrap(),
        leaf1,
    );
    assert_eq!(
        db.get_node(&internal_node_key.gen_child_node_key(1 /* version */, Nibble::from(1)))
            .unwrap(),
        leaf2,
    );
    assert_eq!(db.get_node(&internal_node_key).unwrap(), internal);
    assert_eq!(
        db.get_node(&NodeKey::new_empty_path(1)).unwrap(),
        root_internal,
    );

    // 3. Update leaf2 with new value
    let value2_update = AccountStateBlob::from(vec![5u8, 6u8]);
    let (_root2_hash, batch) = tree
        .put_blob_set(vec![(key2, value2_update.clone())], 2 /* version */)
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert!(tree.get(key2, 0).unwrap().is_none());
    assert_eq!(tree.get(key2, 1).unwrap().unwrap(), value2);
    assert_eq!(tree.get(key2, 2).unwrap().unwrap(), value2_update);

    // Get # of nodes.
    assert_eq!(db.num_nodes(), 8);

    // Purge retired nodes.
    db.purge_stale_nodes(1).unwrap();
    assert_eq!(db.num_nodes(), 7);
    db.purge_stale_nodes(2).unwrap();
    assert_eq!(db.num_nodes(), 4);
    assert_eq!(tree.get(key1, 2).unwrap().unwrap(), value1);
    assert_eq!(tree.get(key2, 2).unwrap().unwrap(), value2_update);
}

#[test]
fn test_batch_insertion() {
    // ```text
    //                             internal(root)
    //                            /        \
    //                       internal       2        <- nibble 0
    //                      /   |   \
    //              internal    3    4               <- nibble 1
    //                 |
    //              internal                         <- nibble 2
    //              /      \
    //        internal      6                        <- nibble 3
    //           |
    //        internal                               <- nibble 4
    //        /      \
    //       1        5                              <- nibble 5
    //
    // Total: 12 nodes
    // ```
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8]);

    let key2 = update_nibble(&key1, 0, 2);
    let value2 = AccountStateBlob::from(vec![2u8]);
    let value2_update = AccountStateBlob::from(vec![22u8]);

    let key3 = update_nibble(&key1, 1, 3);
    let value3 = AccountStateBlob::from(vec![3u8]);

    let key4 = update_nibble(&key1, 1, 4);
    let value4 = AccountStateBlob::from(vec![4u8]);

    let key5 = update_nibble(&key1, 5, 5);
    let value5 = AccountStateBlob::from(vec![5u8]);

    let key6 = update_nibble(&key1, 3, 6);
    let value6 = AccountStateBlob::from(vec![6u8]);

    let batches = vec![
        vec![(key1, value1)],
        vec![(key2, value2)],
        vec![(key3, value3)],
        vec![(key4, value4)],
        vec![(key5, value5)],
        vec![(key6, value6)],
        vec![(key2, value2_update)],
    ];
    let one_batch = batches.iter().flatten().cloned().collect::<Vec<_>>();

    let mut to_verify = one_batch.clone();
    // key2 was updated so we remove it.
    to_verify.remove(1);
    let verify_fn = |tree: &JellyfishMerkleTree<MockTreeStore>, version: Version| {
        to_verify
            .iter()
            .for_each(|(k, v)| assert_eq!(tree.get(*k, version).unwrap().unwrap(), *v))
    };

    // Insert as one batch.
    {
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);

        let (_root, batch) = tree.put_blob_set(one_batch, 0 /* version */).unwrap();
        db.write_tree_update_batch(batch).unwrap();
        verify_fn(&tree, 0);

        // get # of nodes
        assert_eq!(db.num_nodes(), 12);
    }

    // Insert in multiple batches.
    {
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);

        let (_roots, batch) = tree.put_blob_sets(batches, 0 /* first_version */).unwrap();
        db.write_tree_update_batch(batch).unwrap();
        verify_fn(&tree, 6);

        // get # of nodes
        assert_eq!(db.num_nodes(), 26 /* 1 + 3 + 4 + 3 + 8 + 5 + 2 */);

        // Purge retired nodes('p' means purged and 'a' means added).
        // The initial state of the tree at version 0
        // ```test
        //   1(root)
        // ```
        db.purge_stale_nodes(1).unwrap();
        // ```text
        //   1 (p)           internal(a)
        //           ->     /        \
        //                 1(a)       2(a)
        // add 3, prune 1
        // ```
        assert_eq!(db.num_nodes(), 25);
        db.purge_stale_nodes(2).unwrap();
        // ```text
        //     internal(p)             internal(a)
        //    /        \              /        \
        //   1(p)       2   ->   internal(a)    2
        //                       /       \
        //                      1(a)      3(a)
        // add 4, prune 2
        // ```
        assert_eq!(db.num_nodes(), 23);
        db.purge_stale_nodes(3).unwrap();
        // ```text
        //         internal(p)                internal(a)
        //        /        \                 /        \
        //   internal(p)    2   ->     internal(a)     2
        //   /       \                /   |   \
        //  1         3              1    3    4(a)
        // add 3, prune 2
        // ```
        assert_eq!(db.num_nodes(), 21);
        db.purge_stale_nodes(4).unwrap();
        // ```text
        //            internal(p)                         internal(a)
        //           /        \                          /        \
        //     internal(p)     2                    internal(a)    2
        //    /   |   \                            /   |   \
        //   1(p) 3    4           ->      internal(a) 3    4
        //                                     |
        //                                 internal(a)
        //                                     |
        //                                 internal(a)
        //                                     |
        //                                 internal(a)
        //                                 /      \
        //                                1(a)     5(a)
        // add 8, prune 3
        // ```
        assert_eq!(db.num_nodes(), 18);
        db.purge_stale_nodes(5).unwrap();
        // ```text
        //                  internal(p)                             internal(a)
        //                 /        \                              /        \
        //            internal(p)    2                        internal(a)    2
        //           /   |   \                               /   |   \
        //   internal(p) 3    4                      internal(a) 3    4
        //       |                                      |
        //   internal(p)                 ->          internal(a)
        //       |                                   /      \
        //   internal                          internal      6(a)
        //       |                                |
        //   internal                          internal
        //   /      \                          /      \
        //  1        5                        1        5
        // add 5, prune 4
        // ```
        assert_eq!(db.num_nodes(), 14);
        db.purge_stale_nodes(6).unwrap();
        // ```text
        //                         internal(p)                               internal(a)
        //                        /        \                                /        \
        //                   internal       2(p)                       internal       2(a)
        //                  /   |   \                                 /   |   \
        //          internal    3    4                        internal    3    4
        //             |                                         |
        //          internal                      ->          internal
        //          /      \                                  /      \
        //    internal      6                           internal      6
        //       |                                         |
        //    internal                                  internal
        //    /      \                                  /      \
        //   1        5                                1        5
        // add 2, prune 2
        // ```
        assert_eq!(db.num_nodes(), 12);
        verify_fn(&tree, 6);
    }
}

#[test]
fn test_non_existence() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);
    // ```text
    //                     internal(root)
    //                    /        \
    //                internal      2
    //                   |
    //                internal
    //                /      \
    //               1        3
    // Total: 7 nodes
    // ```
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8]);

    let key2 = update_nibble(&key1, 0, 15);
    let value2 = AccountStateBlob::from(vec![2u8]);

    let key3 = update_nibble(&key1, 2, 3);
    let value3 = AccountStateBlob::from(vec![3u8]);

    let (root, batch) = tree
        .put_blob_set(
            vec![
                (key1, value1.clone()),
                (key2, value2.clone()),
                (key3, value3.clone()),
            ],
            0, /* version */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, 0).unwrap().unwrap(), value1);
    assert_eq!(tree.get(key2, 0).unwrap().unwrap(), value2);
    assert_eq!(tree.get(key3, 0).unwrap().unwrap(), value3);
    // get # of nodes
    assert_eq!(db.num_nodes(), 6);

    // test non-existing nodes.
    // 1. Non-existing node at root node
    {
        let non_existing_key = update_nibble(&key1, 0, 1);
        let (value, proof) = tree.get_with_proof(non_existing_key, 0).unwrap();
        assert_eq!(value, None);
        assert!(proof.verify(root, non_existing_key, None).is_ok());
    }
    // 2. Non-existing node at non-root internal node
    {
        let non_existing_key = update_nibble(&key1, 1, 15);
        let (value, proof) = tree.get_with_proof(non_existing_key, 0).unwrap();
        assert_eq!(value, None);
        assert!(proof.verify(root, non_existing_key, None).is_ok());
    }
    // 3. Non-existing node at leaf node
    {
        let non_existing_key = update_nibble(&key1, 2, 4);
        let (value, proof) = tree.get_with_proof(non_existing_key, 0).unwrap();
        assert_eq!(value, None);
        assert!(proof.verify(root, non_existing_key, None).is_ok());
    }
}

#[test]
fn test_put_blob_sets() {
    let mut keys = vec![];
    let mut values = vec![];
    let total_updates = 20;
    for _i in 0..total_updates {
        keys.push(HashValue::random());
        values.push(AccountStateBlob::from(HashValue::random().to_vec()));
    }

    let mut root_hashes_one_by_one = vec![];
    let mut batch_one_by_one = TreeUpdateBatch::default();
    {
        let mut iter = keys.clone().into_iter().zip(values.clone().into_iter());
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);
        for version in 0..10 {
            let mut keyed_blob_set = vec![];
            for _ in 0..total_updates / 10 {
                keyed_blob_set.push(iter.next().unwrap());
            }
            let (root, batch) = tree
                .put_blob_set(keyed_blob_set, version as Version)
                .unwrap();
            db.write_tree_update_batch(batch.clone()).unwrap();
            root_hashes_one_by_one.push(root);
            batch_one_by_one.node_batch.extend(batch.node_batch);
            batch_one_by_one
                .stale_node_index_batch
                .extend(batch.stale_node_index_batch);
            batch_one_by_one.num_new_leaves += batch.num_new_leaves;
            batch_one_by_one.num_stale_leaves += batch.num_stale_leaves;
        }
    }
    {
        let mut iter = keys.into_iter().zip(values.into_iter());
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);
        let mut blob_sets = vec![];
        for _ in 0..10 {
            let mut keyed_blob_set = vec![];
            for _ in 0..total_updates / 10 {
                keyed_blob_set.push(iter.next().unwrap());
            }
            blob_sets.push(keyed_blob_set);
        }
        let (root_hashes, batch) = tree.put_blob_sets(blob_sets, 0 /* version */).unwrap();
        assert_eq!(root_hashes, root_hashes_one_by_one);
        assert_eq!(batch, batch_one_by_one);
    }
}

fn many_keys_get_proof_and_verify_tree_root(seed: &[u8], num_keys: usize) {
    assert!(seed.len() < 32);
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(&seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let mut kvs = vec![];
    for _i in 0..num_keys {
        let key = HashValue::random_with_rng(&mut rng);
        let value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.push((key, value));
    }

    let (root, batch) = tree.put_blob_set(kvs.clone(), 0 /* version */).unwrap();
    db.write_tree_update_batch(batch).unwrap();

    for (k, v) in &kvs {
        let (value, proof) = tree.get_with_proof(*k, 0).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(proof.verify(root, *k, Some(v)).is_ok());
    }
}

#[test]
fn test_1000_keys() {
    let seed: &[_] = &[1, 2, 3, 4];
    many_keys_get_proof_and_verify_tree_root(seed, 1000);
}

fn many_versions_get_proof_and_verify_tree_root(seed: &[u8], num_versions: usize) {
    assert!(seed.len() < 32);
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(&seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let mut kvs = vec![];
    let mut roots = vec![];

    for _i in 0..num_versions {
        let key = HashValue::random_with_rng(&mut rng);
        let value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
        let new_value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.push((key, value.clone(), new_value.clone()));
    }

    for (idx, kvs) in kvs.iter().enumerate() {
        let (root, batch) = tree
            .put_blob_set(vec![(kvs.0, kvs.1.clone())], idx as Version)
            .unwrap();
        roots.push(root);
        db.write_tree_update_batch(batch).unwrap();
    }

    // Update value of all keys
    for (idx, kvs) in kvs.iter().enumerate() {
        let version = (num_versions + idx) as Version;
        let (root, batch) = tree
            .put_blob_set(vec![(kvs.0, kvs.2.clone())], version)
            .unwrap();
        roots.push(root);
        db.write_tree_update_batch(batch).unwrap();
    }

    for (i, (k, v, _)) in kvs.iter().enumerate() {
        let random_version = rng.gen_range(i, i + num_versions);
        let (value, proof) = tree.get_with_proof(*k, random_version as Version).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(proof.verify(roots[random_version], *k, Some(v)).is_ok());
    }

    for (i, (k, _, v)) in kvs.iter().enumerate() {
        let random_version = rng.gen_range(i + num_versions, 2 * num_versions);
        let (value, proof) = tree.get_with_proof(*k, random_version as Version).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(proof.verify(roots[random_version], *k, Some(v)).is_ok());
    }
}

#[test]
fn test_1000_versions() {
    let seed: &[_] = &[1, 2, 3, 4];
    many_versions_get_proof_and_verify_tree_root(seed, 1000);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_get_with_proof1(
        (existent_kvs, nonexistent_keys) in hash_map(
            any::<HashValue>(),
            any::<AccountStateBlob>(),
            1..1000,
        )
            .prop_flat_map(|kvs| {
                let kvs_clone = kvs.clone();
                (
                    Just(kvs),
                    vec(
                        any::<HashValue>().prop_filter(
                            "Make sure these keys do not exist in the tree.",
                            move |key| !kvs_clone.contains_key(key),
                        ),
                        100,
                    ),
                )
            })
    ) {
        let (db, version) = init_mock_db(&existent_kvs);
        let tree = JellyfishMerkleTree::new(&db);

        test_existent_keys_impl(&tree, version, &existent_kvs);
        test_nonexistent_keys_impl(&tree, version, &nonexistent_keys);
    }

    #[test]
    fn test_get_with_proof2(
        key1 in any::<HashValue>()
            .prop_filter(
                "Can't be 0xffffff...",
                |key| *key != HashValue::new([0xff; HashValue::LENGTH]),
            ),
        accounts in vec(any::<AccountStateBlob>(), 2),
    ) {
        let key2 = plus_one(key1);

        let mut kvs = HashMap::new();
        kvs.insert(key1, accounts[0].clone());
        kvs.insert(key2, accounts[1].clone());

        let (db, version) = init_mock_db(&kvs);
        let tree = JellyfishMerkleTree::new(&db);

        test_existent_keys_impl(&tree, version, &kvs);
    }

    #[test]
    fn test_get_range_proof(
        (btree, n) in btree_map(any::<HashValue>(), any::<AccountStateBlob>(), 1..1000)
            .prop_flat_map(|btree| {
                let len = btree.len();
                (Just(btree), 0..len)
            })
    ) {
        let (db, version) = init_mock_db(&btree.clone().into_iter().collect());
        let tree = JellyfishMerkleTree::new(&db);

        let nth_key = *btree.keys().nth(n).unwrap();
        let proof = tree.get_range_proof(nth_key, version).unwrap();
        verify_range_proof(
            tree.get_root_hash(version).unwrap(),
            btree.into_iter().take(n + 1).collect(),
            proof,
        );
    }
}

fn test_existent_keys_impl<'a>(
    tree: &JellyfishMerkleTree<'a, MockTreeStore>,
    version: Version,
    existent_kvs: &HashMap<HashValue, AccountStateBlob>,
) {
    let root_hash = tree.get_root_hash(version).unwrap();

    for (key, value) in existent_kvs {
        let (account, proof) = tree.get_with_proof(*key, version).unwrap();
        assert!(proof.verify(root_hash, *key, account.as_ref()).is_ok());
        assert_eq!(account.unwrap(), *value);
    }
}

fn test_nonexistent_keys_impl<'a>(
    tree: &JellyfishMerkleTree<'a, MockTreeStore>,
    version: Version,
    nonexistent_keys: &[HashValue],
) {
    let root_hash = tree.get_root_hash(version).unwrap();

    for key in nonexistent_keys {
        let (account, proof) = tree.get_with_proof(*key, version).unwrap();
        assert!(proof.verify(root_hash, *key, account.as_ref()).is_ok());
        assert!(account.is_none());
    }
}

/// Checks if we can construct the expected root hash using the entries in the btree and the proof.
fn verify_range_proof(
    expected_root_hash: HashValue,
    btree: BTreeMap<HashValue, AccountStateBlob>,
    proof: SparseMerkleRangeProof,
) {
    // For example, given the following sparse Merkle tree:
    //
    //                   root
    //                  /     \
    //                 /       \
    //                /         \
    //               o           o
    //              / \         / \
    //             a   o       o   h
    //                / \     / \
    //               o   d   e   X
    //              / \         / \
    //             b   c       f   g
    //
    // we transform the keys as follows:
    //   a => 00,
    //   b => 0100,
    //   c => 0101,
    //   d => 011,
    //   e => 100,
    //   X => 101
    //   h => 11
    //
    // Basically, the suffixes that doesn't affect the common prefix of adjacent leaves are
    // discarded. In this example, we assume `btree` has the keys `a` to `e` and the proof has `X`
    // and `h` in the siblings.

    // Now we want to construct a set of key-value pairs that covers the entire set of leaves. For
    // `a` to `e` this is simple -- we just insert them directly into this set. For the rest of the
    // leaves, they are represented by the siblings, so we just make up some keys that make sense.
    // For example, for `X` we just use 101000... (more zeros omitted), because that is one key
    // that would cause `X` to end up in the above position.
    let mut btree1 = BTreeMap::new();
    for (key, blob) in &btree {
        let leaf = LeafNode::new(*key, blob.clone());
        btree1.insert(*key, leaf.hash());
    }
    // Using the above example, `last_proven_key` is `e`. We look at the path from root to `e`.
    // For each 0-bit, there should be a sibling in the proof. And we use the path from root to
    // this position, plus a `1` as the key.
    let last_proven_key = *btree
        .keys()
        .last()
        .expect("We are proving at least one key.");
    for (i, sibling) in last_proven_key
        .iter_bits()
        .enumerate()
        .filter_map(|(i, bit)| if !bit { Some(i) } else { None })
        .zip(proof.right_siblings().iter().rev())
    {
        // This means the `i`-th bit is zero. We take `i` bits from `last_proven_key` and append a
        // one to make up the key for this sibling.
        let mut buf: Vec<_> = last_proven_key.iter_bits().take(i).collect();
        buf.push(true);
        // The rest doesn't matter, because they don't affect the position of the node. We just
        // add zeros.
        buf.resize(HashValue::LENGTH_IN_BITS, false);
        let key = HashValue::from_bit_iter(buf.into_iter()).unwrap();
        btree1.insert(key, *sibling);
    }

    // Now we do the transformation (removing the suffixes) described above.
    let mut kvs = vec![];
    for (key, value) in &btree1 {
        // The length of the common prefix of the previous key and the current key.
        let prev_common_prefix_len =
            prev_key(&btree1, key).map(|pkey| pkey.common_prefix_bits_len(*key));
        // The length of the common prefix of the next key and the current key.
        let next_common_prefix_len =
            next_key(&btree1, key).map(|nkey| nkey.common_prefix_bits_len(*key));

        // We take the longest common prefix of the current key and its neighbors. That's how much
        // we need to keep.
        let len = match (prev_common_prefix_len, next_common_prefix_len) {
            (Some(plen), Some(nlen)) => std::cmp::max(plen, nlen),
            (Some(plen), None) => plen,
            (None, Some(nlen)) => nlen,
            (None, None) => 0,
        };
        let transformed_key: Vec<_> = key.iter_bits().take(len + 1).collect();
        kvs.push((transformed_key, *value));
    }

    assert_eq!(compute_root_hash(kvs), expected_root_hash);
}

/// Computes the root hash of a sparse Merkle tree. `kvs` consists of the entire set of key-value
/// pairs stored in the tree.
fn compute_root_hash(kvs: Vec<(Vec<bool>, HashValue)>) -> HashValue {
    let mut kv_ref = vec![];
    for (key, value) in &kvs {
        kv_ref.push((&key[..], *value));
    }
    compute_root_hash_impl(kv_ref)
}

fn compute_root_hash_impl(kvs: Vec<(&[bool], HashValue)>) -> HashValue {
    assert!(!kvs.is_empty());

    // If there is only one entry, it is the root.
    if kvs.len() == 1 {
        return kvs[0].1;
    }

    // Otherwise the tree has more than one leaves, which means we can find which ones are in the
    // left subtree and which ones are in the right subtree. So we find the first key that starts
    // with a 1-bit.
    let left_hash;
    let right_hash;
    match kvs.iter().position(|(key, _value)| key[0]) {
        Some(0) => {
            // Every key starts with a 1-bit, i.e., they are all in the right subtree.
            left_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
            right_hash = compute_root_hash_impl(reduce(&kvs));
        }
        Some(index) => {
            // Both left subtree and right subtree have some keys.
            left_hash = compute_root_hash_impl(reduce(&kvs[..index]));
            right_hash = compute_root_hash_impl(reduce(&kvs[index..]));
        }
        None => {
            // Every key starts with a 0-bit, i.e., they are all in the left subtree.
            left_hash = compute_root_hash_impl(reduce(&kvs));
            right_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
        }
    }

    SparseMerkleInternalNode::new(left_hash, right_hash).hash()
}

/// Reduces the problem by removing the first bit of every key.
fn reduce<'a>(kvs: &'a [(&[bool], HashValue)]) -> Vec<(&'a [bool], HashValue)> {
    kvs.iter().map(|(key, value)| (&key[1..], *value)).collect()
}

/// Returns the key immediately before `key` in `btree`.
fn prev_key<K, V>(btree: &BTreeMap<K, V>, key: &K) -> Option<K>
where
    K: Clone + Ord,
{
    btree
        .range((Bound::Unbounded, Bound::Excluded(key)))
        .next_back()
        .map(|(k, _v)| k.clone())
}

/// Returns the key immediately after `key` in `btree`.
fn next_key<K, V>(btree: &BTreeMap<K, V>, key: &K) -> Option<K>
where
    K: Clone + Ord,
{
    btree
        .range((Bound::Excluded(key), Bound::Unbounded))
        .next()
        .map(|(k, _v)| k.clone())
}
