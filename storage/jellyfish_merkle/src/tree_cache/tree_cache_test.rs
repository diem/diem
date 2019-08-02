// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{mock_tree_store::MockTreeStore, node_type::Node, NodeKey};
use crypto::HashValue;
use sparse_merkle::nibble_path::NibblePath;
use types::account_state_blob::AccountStateBlob;

fn random_leaf_with_key(next_version: Version) -> (Node, NodeKey) {
    let address = HashValue::random();
    let node = Node::new_leaf(
        address,
        AccountStateBlob::from(HashValue::random().to_vec()),
    );
    let node_key = NodeKey::new(next_version, NibblePath::new(address.to_vec()));
    (node, node_key)
}

#[test]
fn test_get_node() {
    let next_version = 0;
    let db = MockTreeStore::default();
    let cache = TreeCache::new(&db, next_version);

    let (node, node_key) = random_leaf_with_key(next_version);
    db.put_node(node_key.clone(), node.clone()).unwrap();

    assert_eq!(cache.get_node(&node_key).unwrap(), node);
}

#[test]
fn test_root_node() {
    let next_version = 0;
    let db = MockTreeStore::default();
    let mut cache = TreeCache::new(&db, next_version);
    assert_eq!(cache.get_root_node().unwrap(), None);

    let (node, node_key) = random_leaf_with_key(next_version);
    db.put_node(node_key.clone(), node.clone()).unwrap();
    cache.set_root_node_key(Some(node_key));

    assert_eq!(cache.get_root_node().unwrap().unwrap(), node);
}

#[test]
fn test_freeze_with_delete() {
    let next_version = 0;
    let db = MockTreeStore::default();
    let mut cache = TreeCache::new(&db, next_version);

    assert_eq!(cache.get_root_node().unwrap(), None);

    let (node1, node1_key) = random_leaf_with_key(next_version);
    cache.put_node(node1_key.clone(), node1.clone()).unwrap();
    let (node2, node2_key) = random_leaf_with_key(next_version);
    cache.put_node(node2_key.clone(), node2.clone()).unwrap();
    assert_eq!(cache.get_node(&node1_key).unwrap(), node1);
    assert_eq!(cache.get_node(&node2_key).unwrap(), node2);
    cache.freeze().unwrap();
    assert_eq!(cache.get_node(&node1_key).unwrap(), node1);
    assert_eq!(cache.get_node(&node2_key).unwrap(), node2);

    cache.delete_node(&node1_key);
    cache.freeze().unwrap();
    let (_, update_batch) = cache.into();
    let (node_batch, retire_log_batch) = update_batch.into();
    assert_eq!(node_batch.len(), 2);
    assert_eq!(retire_log_batch.len(), 1);
}
