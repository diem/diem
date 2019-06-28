// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    mock_tree_store::MockTreeStore,
    node_type::{LeafNode, Node},
};
use crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};

#[test]
fn test_get_node() {
    let db = MockTreeStore::default();
    let cache = TreeCache::new(
        &db,
        *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root_node_hash */
        0,                               /* next_version */
    );

    let address = HashValue::random();
    let value_hash = HashValue::random();
    let leaf_node = Node::Leaf(LeafNode::new(address, value_hash));

    db.put_node(leaf_node.hash(), leaf_node.clone()).unwrap();
    assert_eq!(cache.get_node(leaf_node.hash()).unwrap(), leaf_node);
}

#[test]
fn test_root_node() {
    let db = MockTreeStore::default();
    let mut cache = TreeCache::new(
        &db,
        *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root_node_hash */
        0,                               /* next_version */
    );

    assert_eq!(cache.get_root_node().unwrap(), None);

    let address = HashValue::random();
    let value_hash = HashValue::random();
    let leaf_node = Node::Leaf(LeafNode::new(address, value_hash));

    db.put_node(leaf_node.hash(), leaf_node.clone()).unwrap();
    cache.set_root_hash(leaf_node.hash());

    assert_eq!(cache.get_root_node().unwrap().unwrap(), leaf_node);
}

#[test]
fn test_duplicate_blob() {
    let db = MockTreeStore::default();
    let mut cache = TreeCache::new(
        &db,
        *SPARSE_MERKLE_PLACEHOLDER_HASH, /* root_node_hash */
        0,                               /* next_version */
    );

    let blob = AccountStateBlob::from(vec![0u8]);
    let blob_hash = HashValue::random();
    cache.put_blob(blob_hash, blob.clone()).unwrap();
    cache.put_blob(blob_hash, blob.clone()).unwrap();
    assert_eq!(cache.get_blob(blob_hash).unwrap(), blob);
    cache.delete_blob(blob_hash);
    assert_eq!(cache.get_blob(blob_hash).unwrap(), blob);
    cache.delete_blob(blob_hash);
    assert!(cache.get_blob(blob_hash).is_err());
}
