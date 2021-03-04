// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{mock_tree_store::MockTreeStore, node_type::LeafNode, JellyfishMerkleTree};
use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use diem_types::{
    proof::{SparseMerkleInternalNode, SparseMerkleRangeProof},
    transaction::Version,
};
use proptest::{
    collection::{btree_map, hash_map, vec},
    prelude::*,
};
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
};

#[derive(
    Arbitrary,
    BCSCryptoHash,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    CryptoHasher,
)]
pub(crate) struct ValueBlob(Vec<u8>);

impl From<Vec<u8>> for ValueBlob {
    fn from(blob: Vec<u8>) -> Self {
        Self(blob)
    }
}

impl crate::Value for ValueBlob {}
impl crate::TestValue for ValueBlob {}

/// Computes the key immediately after `key`.
pub fn plus_one(key: HashValue) -> HashValue {
    assert_ne!(key, HashValue::new([0xff; HashValue::LENGTH]));

    let mut buf = key.to_vec();
    for i in (0..HashValue::LENGTH).rev() {
        if buf[i] == 255 {
            buf[i] = 0;
        } else {
            buf[i] += 1;
            break;
        }
    }
    HashValue::from_slice(&buf).unwrap()
}

/// Initializes a DB with a set of key-value pairs by inserting one key at each version.
pub fn init_mock_db<V>(kvs: &HashMap<HashValue, V>) -> (MockTreeStore<V>, Version)
where
    V: crate::TestValue,
{
    assert!(!kvs.is_empty());

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    for (i, (key, value)) in kvs.iter().enumerate() {
        let (_root_hash, write_batch) = tree
            .put_value_set(vec![(*key, value.clone())], i as Version)
            .unwrap();
        db.write_tree_update_batch(write_batch).unwrap();
    }

    (db, (kvs.len() - 1) as Version)
}

pub fn arb_existent_kvs_and_nonexistent_keys<V: crate::TestValue>(
    num_kvs: usize,
    num_non_existing_keys: usize,
) -> impl Strategy<Value = (HashMap<HashValue, V>, Vec<HashValue>)> {
    hash_map(any::<HashValue>(), any::<V>(), 1..num_kvs).prop_flat_map(move |kvs| {
        let kvs_clone = kvs.clone();
        (
            Just(kvs),
            vec(
                any::<HashValue>().prop_filter(
                    "Make sure these keys do not exist in the tree.",
                    move |key| !kvs_clone.contains_key(key),
                ),
                num_non_existing_keys,
            ),
        )
    })
}

pub fn test_get_with_proof<V: crate::TestValue>(
    (existent_kvs, nonexistent_keys): (HashMap<HashValue, V>, Vec<HashValue>),
) {
    let (db, version) = init_mock_db(&existent_kvs);
    let tree = JellyfishMerkleTree::new(&db);

    test_existent_keys_impl(&tree, version, &existent_kvs);
    test_nonexistent_keys_impl(&tree, version, &nonexistent_keys);
}

pub fn arb_kv_pair_with_distinct_last_nibble<V: crate::TestValue>(
) -> impl Strategy<Value = ((HashValue, V), (HashValue, V))> {
    (
        any::<HashValue>().prop_filter("Can't be 0xffffff...", |key| {
            *key != HashValue::new([0xff; HashValue::LENGTH])
        }),
        vec(any::<V>(), 2),
    )
        .prop_map(|(key1, accounts)| {
            let key2 = plus_one(key1);
            ((key1, accounts[0].clone()), (key2, accounts[1].clone()))
        })
}

pub fn test_get_with_proof_with_distinct_last_nibble<V: crate::TestValue>(
    (kv1, kv2): ((HashValue, V), (HashValue, V)),
) {
    let mut kvs = HashMap::new();
    kvs.insert(kv1.0, kv1.1);
    kvs.insert(kv2.0, kv2.1);

    let (db, version) = init_mock_db(&kvs);
    let tree = JellyfishMerkleTree::new(&db);

    test_existent_keys_impl(&tree, version, &kvs);
}

pub fn arb_tree_with_index<V: crate::TestValue>(
    tree_size: usize,
) -> impl Strategy<Value = (BTreeMap<HashValue, V>, usize)> {
    btree_map(any::<HashValue>(), any::<V>(), 1..tree_size).prop_flat_map(|btree| {
        let len = btree.len();
        (Just(btree), 0..len)
    })
}

pub fn test_get_range_proof<V: crate::TestValue>((btree, n): (BTreeMap<HashValue, V>, usize)) {
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

fn test_existent_keys_impl<'a, V: crate::TestValue>(
    tree: &JellyfishMerkleTree<'a, MockTreeStore<V>, V>,
    version: Version,
    existent_kvs: &HashMap<HashValue, V>,
) {
    let root_hash = tree.get_root_hash(version).unwrap();

    for (key, value) in existent_kvs {
        let (account, proof) = tree.get_with_proof(*key, version).unwrap();
        assert!(proof.verify(root_hash, *key, account.as_ref()).is_ok());
        assert_eq!(account.unwrap(), *value);
    }
}

fn test_nonexistent_keys_impl<'a, V: crate::TestValue>(
    tree: &JellyfishMerkleTree<'a, MockTreeStore<V>, V>,
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
fn verify_range_proof<V: crate::TestValue>(
    expected_root_hash: HashValue,
    btree: BTreeMap<HashValue, V>,
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
    for (key, value) in &btree {
        let leaf = LeafNode::new(*key, value.clone());
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

fn next_key<K, V>(btree: &BTreeMap<K, V>, key: &K) -> Option<K>
where
    K: Clone + Ord,
{
    btree
        .range((Bound::Excluded(key), Bound::Unbounded))
        .next()
        .map(|(k, _v)| k.clone())
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
