// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use mock_tree_store::MockTreeStore;
use rand::{rngs::StdRng, Rng, SeedableRng};
use types::proof::verify_sparse_merkle_element;

fn modify(original_key: &HashValue, n: usize, value: u8) -> HashValue {
    let mut key = original_key.to_vec();
    key[n] = value;
    HashValue::from_slice(&key).unwrap()
}

#[test]
fn test_insert_to_empty_tree() {
    let db = MockTreeStore::default();
    let tree = SparseMerkleTree::new(&db);

    // Tree is initially empty. Root is a null node. We'll insert a key-value pair which creates a
    // leaf node.
    let key = HashValue::random();
    let value = AccountStateBlob::from(vec![1u8, 2u8, 3u8, 4u8]);

    let (new_root, batch) = tree
        .put_blob_set(
            vec![(key, value.clone())],
            0,                               /* version */
            *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
        )
        .unwrap();
    assert!(batch.retired_record_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();

    assert_eq!(tree.get(key, new_root).unwrap().unwrap(), value);
}

#[test]
fn test_insert_at_leaf_with_branch_created() {
    let db = MockTreeStore::default();
    let tree = SparseMerkleTree::new(&db);

    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (root1, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone())],
            0,                               /* version */
            *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
        )
        .unwrap();
    assert!(batch.retired_record_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);

    // Insert at the previous leaf node. Should generate a branch node at root.
    // Change the 1st nibble to 15.
    let key2 = modify(&key1, 0, 0xf0);
    let value2 = AccountStateBlob::from(vec![3u8, 4u8]);

    let (root2, batch) = tree
        .put_blob_set(
            vec![(key2, value2.clone())],
            1,     /* version */
            root1, /* root hash being based on */
        )
        .unwrap();
    assert!(batch.retired_record_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);
    assert!(tree.get(key2, root1).unwrap().is_none());
    assert_eq!(tree.get(key2, root2).unwrap().unwrap(), value2);

    // get # of nodes
    assert_eq!(db.num_nodes(), 3);
    assert_eq!(db.num_blobs(), 2);

    let leaf1 = LeafNode::new(key1, value1.hash());
    let leaf2 = LeafNode::new(key2, value2.hash());
    let mut branch = BranchNode::default();
    branch.set_child(0, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(15, (leaf2.hash(), true /* is_leaf */));
    assert_eq!(db.get_node(root1).unwrap(), leaf1.into());
    assert_eq!(db.get_node(leaf2.hash()).unwrap(), leaf2.into());
    assert_eq!(db.get_node(root2).unwrap(), branch.into());
}

#[test]
fn test_insert_at_leaf_with_extension_and_branch_created() {
    let db = MockTreeStore::default();
    let tree = SparseMerkleTree::new(&db);

    // 1. Insert the first leaf into empty tree
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (root1, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone())],
            0,                               /* version */
            *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);

    // 2. Insert at the previous leaf node. Should generate a branch node at root.
    // Change the 2nd nibble to 15.
    let key2 = modify(&key1, 0, 0x01);
    let value2 = AccountStateBlob::from(vec![3u8, 4u8]);

    let (root2, batch) = tree
        .put_blob_set(
            vec![(key2, value2.clone())],
            1,     /* version */
            root1, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);
    assert!(tree.get(key2, root1).unwrap().is_none());
    assert_eq!(tree.get(key2, root2).unwrap().unwrap(), value2);

    assert_eq!(db.num_nodes(), 4);
    assert_eq!(db.num_blobs(), 2);

    let leaf1 = LeafNode::new(key1, value1.hash());
    let leaf2 = LeafNode::new(key2, value2.hash());
    let mut branch = BranchNode::default();
    branch.set_child(0, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(1, (leaf2.hash(), true /* is_leaf */));
    let extension = ExtensionNode::new(NibblePath::new_odd(vec![0x00]), branch.hash());
    assert_eq!(db.get_node(root1).unwrap(), leaf1.clone().into());
    assert_eq!(
        db.get_node(branch.child(1).unwrap()).unwrap(),
        leaf2.clone().into()
    );
    assert_eq!(
        db.get_node(extension.child()).unwrap(),
        branch.clone().into()
    );
    assert_eq!(db.get_node(root2).unwrap(), extension.clone().into());

    // 3. Update leaf2 with new value
    let value2_update = AccountStateBlob::from(vec![5u8, 6u8]);
    let (root3, batch) = tree
        .put_blob_set(
            vec![(key2, value2_update.clone())],
            2,     /* version */
            root2, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert!(tree.get(key2, root1).unwrap().is_none());
    assert_eq!(tree.get(key2, root2).unwrap().unwrap(), value2);
    assert_eq!(tree.get(key2, root3).unwrap().unwrap(), value2_update);

    // Get # of nodes.
    assert_eq!(db.num_nodes(), 7);
    assert_eq!(db.num_blobs(), 3);

    // Purge retired nodes.
    db.prune_retired_records(1).unwrap();
    assert_eq!(db.num_nodes(), 7);
    assert_eq!(db.num_blobs(), 3);
    db.prune_retired_records(2).unwrap();
    assert_eq!(db.num_nodes(), 4);
    assert_eq!(db.num_blobs(), 2);
    assert_eq!(tree.get(key1, root3).unwrap().unwrap(), value1);
    assert_eq!(tree.get(key2, root3).unwrap().unwrap(), value2_update);
}

fn setup_extension_case(db: &MockTreeStore, n: usize) -> (HashValue, HashValue) {
    assert!(n / 2 < HashValue::LENGTH);
    let tree = SparseMerkleTree::new(db);
    let key1 = HashValue::new([0xffu8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![0xff, 0xff]);

    // Change the n-th nibble to 0xE so it results in an extension node with num_nibbles == n;
    // if n == 0, no extension node will be created.
    let key2 = modify(&key1, n / 2, if n % 2 == 0 { 0xef } else { 0xfe });
    let value2 = AccountStateBlob::from(vec![0xee, 0xee]);

    let (root, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone()), (key2, value2.clone())],
            0,                               /* version */
            *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(db.num_nodes(), 4);
    assert_eq!(db.num_blobs(), 2);

    let leaf1 = LeafNode::new(key1, value1.hash());
    let leaf2 = LeafNode::new(key2, value2.hash());
    let mut branch = BranchNode::default();
    branch.set_child(15, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(14, (leaf2.hash(), true /* is_leaf */));
    let branch_hash = branch.hash();
    if n == 0 {
        assert_eq!(root, branch_hash)
    } else {
        match db.get_node(root).unwrap() {
            Node::Extension(extension) => assert_eq!(extension.child(), branch_hash),
            _ => unreachable!(),
        }
    }
    (root, branch_hash)
}

#[test]
fn test_insert_at_extension_fork_at_begining() {
    let db = MockTreeStore::default();
    let (root, extension_child_hash) = setup_extension_case(&db, 6);
    let tree = SparseMerkleTree::new(&db);

    let key1 = HashValue::new([0x00; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (root1, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone())],
            1,    /* version */
            root, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();

    let extension_after_fork = ExtensionNode::new(
        NibblePath::new_odd(vec![0xff, 0xff, 0xf0]),
        extension_child_hash,
    );
    let leaf1 = LeafNode::new(key1, value1.hash());
    let mut branch = BranchNode::default();
    branch.set_child(0, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(15, (extension_after_fork.hash(), false /* is_leaf */));

    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);
    assert_eq!(db.get_node(branch.child(0).unwrap()).unwrap(), leaf1.into());
    assert_eq!(
        db.get_node(branch.child(15).unwrap()).unwrap(),
        extension_after_fork.into()
    );
    assert_eq!(db.get_node(root1).unwrap(), branch.into());
    assert_eq!(db.num_nodes(), 7);
    assert_eq!(db.num_blobs(), 3);

    // Purge retired nodes. (The old extension should be gone.)
    db.prune_retired_records(1).unwrap();
    assert_eq!(db.num_nodes(), 6);
    assert_eq!(db.num_blobs(), 3);
}

#[test]
fn test_insert_at_extension_fork_in_the_middle() {
    let db = MockTreeStore::default();
    let (root, extension_child_hash) = setup_extension_case(&db, 5);
    let tree = SparseMerkleTree::new(&db);

    let key1 = modify(&HashValue::new([0xff; HashValue::LENGTH]), 1, 0x00);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (root1, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone())],
            1,    /* version */
            root, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();

    let extension_after_fork =
        ExtensionNode::new(NibblePath::new(vec![0xff]), extension_child_hash);
    let leaf1 = LeafNode::new(key1, value1.hash());
    let mut branch = BranchNode::default();
    branch.set_child(0, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(15, (extension_after_fork.hash(), false /* is_leaf */));
    let extension_before_fork = ExtensionNode::new(NibblePath::new(vec![0xff]), branch.hash());

    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);
    assert_eq!(db.get_node(branch.child(0).unwrap()).unwrap(), leaf1.into());
    assert_eq!(
        db.get_node(branch.child(15).unwrap()).unwrap(),
        extension_after_fork.into()
    );
    assert_eq!(
        db.get_node(extension_before_fork.child()).unwrap(),
        branch.into()
    );
    assert_eq!(db.get_node(root1).unwrap(), extension_before_fork.into());
    assert_eq!(db.num_nodes(), 8);
    assert_eq!(db.num_blobs(), 3);

    // Purge retired nodes. (The old extension should be gone.)
    db.prune_retired_records(1).unwrap();
    assert_eq!(db.num_nodes(), 7);
    assert_eq!(db.num_blobs(), 3);
}

#[test]
fn test_insert_at_extension_fork_at_end() {
    let db = MockTreeStore::default();
    let (root, extension_child_hash) = setup_extension_case(&db, 4);
    let tree = SparseMerkleTree::new(&db);

    let key1 = modify(&HashValue::new([0xff; HashValue::LENGTH]), 1, 0xf0);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (root1, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone())],
            1,    /* version */
            root, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();

    let leaf1 = LeafNode::new(key1, value1.hash());
    let mut branch = BranchNode::default();

    branch.set_child(0, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(15, (extension_child_hash, false /* is_leaf */));
    let extension_before_fork =
        ExtensionNode::new(NibblePath::new_odd(vec![0xff, 0xf0]), branch.hash());

    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);
    assert_eq!(db.get_node(branch.child(0).unwrap()).unwrap(), leaf1.into());
    assert_eq!(
        db.get_node(extension_before_fork.child()).unwrap(),
        branch.into()
    );
    assert_eq!(db.get_node(root1).unwrap(), extension_before_fork.into());
    assert_eq!(db.num_nodes(), 7);
    assert_eq!(db.num_blobs(), 3);

    // Purge retired nodes. (The old extension should be gone.)
    db.prune_retired_records(1).unwrap();
    assert_eq!(db.num_nodes(), 6);
    assert_eq!(db.num_blobs(), 3);
}

#[test]
fn test_insert_at_extension_fork_at_only_nibble() {
    let db = MockTreeStore::default();
    let (root, branch_child_hash) = setup_extension_case(&db, 1);
    let tree = SparseMerkleTree::new(&db);

    let key1 = HashValue::new([0x00; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8, 2u8]);

    let (root1, batch) = tree
        .put_blob_set(
            vec![(key1, value1.clone())],
            1,    /* version */
            root, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();

    let leaf1 = LeafNode::new(key1, value1.hash());
    let mut branch = BranchNode::default();
    branch.set_child(0, (leaf1.hash(), true /* is_leaf */));
    branch.set_child(15, (branch_child_hash, false /* is_leaf */));

    assert_eq!(tree.get(key1, root1).unwrap().unwrap(), value1);
    assert_eq!(db.get_node(branch.child(0).unwrap()).unwrap(), leaf1.into());
    assert_eq!(db.get_node(root1).unwrap(), branch.into());
    assert_eq!(db.num_nodes(), 6);
    assert_eq!(db.num_blobs(), 3);

    // Purge retired nodes. (The old extension should be gone.)
    db.prune_retired_records(1).unwrap();
    assert_eq!(db.num_nodes(), 5);
    assert_eq!(db.num_blobs(), 3);
}

#[test]
fn test_batch_insertion() {
    // ```text
    //                              branch(root)
    //                            /        \
    //                        branch        2
    //                      /   |   \
    //             extension    3    4
    //                 |
    //               branch
    //              /      \
    //       extension      6
    //           |
    //         branch
    //        /      \
    //       1        5
    //
    // Total: 12 nodes, 6 blobs
    // ```
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8]);

    let key2 = modify(&key1, 0, 0xf0);
    let value2 = AccountStateBlob::from(vec![2u8]);
    let value2_update = AccountStateBlob::from(vec![22u8]);

    let key3 = modify(&key1, 0, 0x03);
    let value3 = AccountStateBlob::from(vec![3u8]);

    let key4 = modify(&key1, 0, 0x04);
    let value4 = AccountStateBlob::from(vec![4u8]);

    let key5 = modify(&key1, 5, 0x05);
    let value5 = AccountStateBlob::from(vec![5u8]);

    let key6 = modify(&key1, 3, 0x06);
    let value6 = AccountStateBlob::from(vec![6u8]);

    let batches = vec![
        vec![(key1, value1.clone())],
        vec![(key2, value2.clone())],
        vec![(key3, value3.clone())],
        vec![(key4, value4.clone())],
        vec![(key5, value5.clone())],
        vec![(key6, value6.clone())],
        vec![(key2, value2_update.clone())],
    ];
    let one_batch = batches.iter().flatten().cloned().collect::<Vec<_>>();

    let mut to_verify = one_batch.clone();
    to_verify.remove(1);
    let verify_fn = |tree: &SparseMerkleTree<MockTreeStore>, root: HashValue| {
        to_verify
            .iter()
            .for_each(|(k, v)| assert_eq!(tree.get(*k, root).unwrap().unwrap(), *v))
    };

    // Insert as one batch.
    {
        let db = MockTreeStore::default();
        let tree = SparseMerkleTree::new(&db);

        let (root, batch) = tree
            .put_blob_set(
                one_batch,
                0,                               /* version */
                *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
            )
            .unwrap();
        db.write_tree_update_batch(batch).unwrap();
        verify_fn(&tree, root);

        // get # of nodes
        assert_eq!(db.num_nodes(), 12);
        assert_eq!(db.num_blobs(), 6);
        verify_fn(&tree, root);
    }

    // Insert in multiple batches.
    {
        let db = MockTreeStore::default();
        let tree = SparseMerkleTree::new(&db);

        let (roots, batch) = tree
            .put_blob_sets(
                batches,
                0,                               /* first_version */
                *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
            )
            .unwrap();
        let root = *roots.last().unwrap();
        db.write_tree_update_batch(batch).unwrap();
        verify_fn(&tree, root);

        // get # of nodes
        assert_eq!(db.num_nodes(), 22);
        assert_eq!(db.num_blobs(), 7);

        // Purge retired nodes.
        db.prune_retired_records(1).unwrap();
        assert_eq!(db.num_nodes(), 22);
        db.prune_retired_records(2).unwrap();
        assert_eq!(db.num_nodes(), 21);
        db.prune_retired_records(3).unwrap();
        assert_eq!(db.num_nodes(), 19);
        db.prune_retired_records(4).unwrap();
        assert_eq!(db.num_nodes(), 17);
        db.prune_retired_records(5).unwrap();
        assert_eq!(db.num_nodes(), 14);
        assert_eq!(db.num_blobs(), 7); // no blobs purged till this point
        db.prune_retired_records(6).unwrap();
        assert_eq!(db.num_nodes(), 12);
        assert_eq!(db.num_blobs(), 6); // and the old value2 is gone here
        verify_fn(&tree, root);
    }
}

#[test]
fn test_non_existence() {
    let db = MockTreeStore::default();
    let tree = SparseMerkleTree::new(&db);
    // ```text
    //                   branch(root)
    //                    /        \
    //               extension      2
    //                   |
    //                branch
    //               /      \
    //              1        3
    // Total: 7 nodes, 3 blobs
    // ```
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(vec![1u8]);

    let key2 = modify(&key1, 0, 0xf0);
    let value2 = AccountStateBlob::from(vec![2u8]);

    let key3 = modify(&key1, 1, 0x03);
    let value3 = AccountStateBlob::from(vec![3u8]);

    let (root, batch) = tree
        .put_blob_set(
            vec![
                (key1, value1.clone()),
                (key2, value2.clone()),
                (key3, value3.clone()),
            ],
            0,                               /* version */
            *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(key1, root).unwrap().unwrap(), value1);
    assert_eq!(tree.get(key2, root).unwrap().unwrap(), value2);
    assert_eq!(tree.get(key3, root).unwrap().unwrap(), value3);
    // get # of nodes
    assert_eq!(db.num_nodes(), 6);
    assert_eq!(db.num_blobs(), 3);

    // test non-existing nodes.
    // 1. Non-existing node at branch node
    {
        let non_existing_key = modify(&key1, 0, 0x10);
        let (value, proof) = tree.get_with_proof(non_existing_key, root).unwrap();
        assert_eq!(value, None);
        assert!(verify_sparse_merkle_element(root, non_existing_key, &None, &proof).is_ok());
    }
    // 2. Non-existing node at extension node
    {
        let non_existing_key = modify(&key1, 1, 0x30);
        let (value, proof) = tree.get_with_proof(non_existing_key, root).unwrap();
        assert_eq!(value, None);
        assert!(verify_sparse_merkle_element(root, non_existing_key, &None, &proof).is_ok());
    }
    // 3. Non-existing node at leaf node
    {
        let non_existing_key = modify(&key1, 10, 0x01);
        let (value, proof) = tree.get_with_proof(non_existing_key, root).unwrap();
        assert_eq!(value, None);
        assert!(verify_sparse_merkle_element(root, non_existing_key, &None, &proof).is_ok());
    }
}

#[test]
fn test_put_blob_sets() {
    let mut keys = vec![];
    let mut values = vec![];;
    for _i in 0..100 {
        keys.push(HashValue::random());
        values.push(AccountStateBlob::from(HashValue::random().to_vec()));
    }

    let mut root_hashes_one_by_one = vec![];
    let mut batch_one_by_one = TreeUpdateBatch::default();
    {
        let mut iter = keys.clone().into_iter().zip(values.clone().into_iter());
        let mut root = *SPARSE_MERKLE_PLACEHOLDER_HASH;
        let db = MockTreeStore::default();
        let tree = SparseMerkleTree::new(&db);
        for version in 0..10 {
            let mut keyed_blob_set = vec![];
            for _ in 0..10 {
                keyed_blob_set.push(iter.next().unwrap());
            }
            let (new_root, batch) = tree
                .put_blob_set(
                    keyed_blob_set,
                    version as Version,
                    root, /* root hash being based on */
                )
                .unwrap();
            root = new_root;
            db.write_tree_update_batch(batch.clone()).unwrap();
            root_hashes_one_by_one.push(root);
            batch_one_by_one.node_batch.extend(batch.node_batch);
            batch_one_by_one.blob_batch.extend(batch.blob_batch);
            batch_one_by_one
                .retired_record_batch
                .extend(batch.retired_record_batch);
        }
    }
    {
        let mut iter = keys.into_iter().zip(values.into_iter());
        let root = *SPARSE_MERKLE_PLACEHOLDER_HASH;
        let db = MockTreeStore::default();
        let tree = SparseMerkleTree::new(&db);
        let mut blob_sets = vec![];
        for _ in 0..10 {
            let mut keyed_blob_set = vec![];
            for _ in 0..10 {
                keyed_blob_set.push(iter.next().unwrap());
            }
            blob_sets.push(keyed_blob_set);
        }
        let (root_hashes, batch) = tree
            .put_blob_sets(
                blob_sets, 0,    /* version */
                root, /* root hash being based on */
            )
            .unwrap();
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
    let tree = SparseMerkleTree::new(&db);

    let mut kvs = vec![];
    for _i in 0..num_keys {
        let key = HashValue::random_with_rng(&mut rng);
        let value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.push((key, value));
    }

    let (root, batch) = tree
        .put_blob_set(
            kvs.clone(),
            0,                               /* version */
            *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root hash being based on */
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();

    for (k, v) in &kvs {
        let (value, proof) = tree.get_with_proof(*k, root).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(verify_sparse_merkle_element(root, *k, &Some(v.clone()), &proof).is_ok());
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
    let tree = SparseMerkleTree::new(&db);

    let mut kvs = vec![];
    let mut roots = vec![];
    let mut prev_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;

    for _i in 0..num_versions {
        let key = HashValue::random_with_rng(&mut rng);
        let value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
        let new_value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.push((key, value.clone(), new_value.clone()));
    }

    for (idx, kvs) in kvs.iter().enumerate() {
        let (new_root, batch) = tree
            .put_blob_set(vec![(kvs.0, kvs.1.clone())], idx as Version, prev_root)
            .unwrap();
        roots.push(new_root);
        prev_root = new_root;
        db.write_tree_update_batch(batch).unwrap();
    }

    // Update value of all keys
    for (idx, kvs) in kvs.iter().enumerate() {
        let version = (num_versions + idx) as Version;
        let (new_root, batch) = tree
            .put_blob_set(vec![(kvs.0, kvs.2.clone())], version, prev_root)
            .unwrap();
        roots.push(new_root);
        prev_root = new_root;
        db.write_tree_update_batch(batch).unwrap();
    }

    for (i, (k, v, _)) in kvs.iter().enumerate() {
        let random_version = rng.gen_range(i, i + num_versions);
        let (value, proof) = tree.get_with_proof(*k, roots[random_version]).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(
            verify_sparse_merkle_element(roots[random_version], *k, &Some(v.clone()), &proof)
                .is_ok()
        );
    }

    for (i, (k, _, v)) in kvs.iter().enumerate() {
        let random_version = rng.gen_range(i + num_versions, 2 * num_versions);
        let (value, proof) = tree.get_with_proof(*k, roots[random_version]).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(
            verify_sparse_merkle_element(roots[random_version], *k, &Some(v.clone()), &proof)
                .is_ok()
        );
    }
}

#[test]
fn test_1000_versions() {
    let seed: &[_] = &[1, 2, 3, 4];
    many_versions_get_proof_and_verify_tree_root(seed, 1000);
}
