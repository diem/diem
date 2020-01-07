// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    node::{LeafNode, LeafValue, SparseMerkleNode},
    AccountState, ProofRead, SparseMerkleTree,
};
use libra_crypto::{
    hash::{CryptoHash, TestOnlyHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleProof};
use std::{collections::HashMap, sync::Arc};

fn hash_internal(left_child: HashValue, right_child: HashValue) -> HashValue {
    libra_types::proof::SparseMerkleInternalNode::new(left_child, right_child).hash()
}

fn hash_leaf(key: HashValue, value_hash: HashValue) -> HashValue {
    libra_types::proof::SparseMerkleLeafNode::new(key, value_hash).hash()
}

#[derive(Default)]
struct ProofReader(HashMap<HashValue, SparseMerkleProof>);

impl ProofReader {
    fn new(key_with_proof: Vec<(HashValue, SparseMerkleProof)>) -> Self {
        ProofReader(key_with_proof.into_iter().collect())
    }
}

impl ProofRead for ProofReader {
    fn get_proof(&self, key: HashValue) -> Option<&SparseMerkleProof> {
        self.0.get(&key)
    }
}

#[test]
fn test_construct_subtree_zero_siblings() {
    let node_hash = HashValue::new([1; HashValue::LENGTH]);
    let node = SparseMerkleNode::new_subtree(node_hash);
    let subtree_node =
        SparseMerkleTree::construct_subtree(std::iter::empty(), std::iter::empty(), Arc::new(node));
    let smt = SparseMerkleTree { root: subtree_node };
    assert_eq!(smt.root_hash(), node_hash);
}

#[test]
fn test_construct_subtree_three_siblings() {
    //                x
    //               / \
    //      [4; 32] c   y
    //                 / \
    //                z   b [3; 32]
    //               / \
    //           node   a [2; 32]
    let key = b"hello".test_only_hash();
    let blob = AccountStateBlob::from(b"world".to_vec());
    let leaf_hash = hash_leaf(key, blob.hash());
    let node = SparseMerkleNode::new_leaf(key, LeafValue::BlobHash(blob.hash()));
    let bits = vec![false, false, true];
    let a_hash = HashValue::new([2; HashValue::LENGTH]);
    let b_hash = HashValue::new([3; HashValue::LENGTH]);
    let c_hash = HashValue::new([4; HashValue::LENGTH]);
    let siblings = vec![a_hash, b_hash, c_hash]
        .into_iter()
        .map(|hash| Arc::new(SparseMerkleNode::new_subtree(hash)));
    let subtree_node =
        SparseMerkleTree::construct_subtree(bits.into_iter(), siblings, Arc::new(node));
    let smt = SparseMerkleTree { root: subtree_node };

    let z_hash = hash_internal(leaf_hash, a_hash);
    let y_hash = hash_internal(z_hash, b_hash);
    let root_hash = hash_internal(c_hash, y_hash);
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
#[should_panic]
fn test_construct_subtree_panic() {
    let node_hash = HashValue::new([1; HashValue::LENGTH]);
    let node = SparseMerkleNode::new_subtree(node_hash);
    let _subtree_node = SparseMerkleTree::construct_subtree(
        std::iter::once(true),
        std::iter::empty(),
        Arc::new(node),
    );
}

#[test]
fn test_construct_subtree_with_new_leaf_override_existing_leaf() {
    let key = b"hello".test_only_hash();
    let old_blob = AccountStateBlob::from(b"old_old_old".to_vec());
    let new_blob = AccountStateBlob::from(b"new_new_new".to_vec());

    let existing_leaf = LeafNode::new(key, LeafValue::BlobHash(old_blob.hash()));

    let subtree = SparseMerkleTree::construct_subtree_with_new_leaf(
        key,
        new_blob.clone(),
        &existing_leaf,
        /* distance_from_root_to_existing_leaf = */ 3,
    );
    let smt = SparseMerkleTree { root: subtree };

    let new_blob_hash = new_blob.hash();
    let root_hash = hash_leaf(key, new_blob_hash);
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
fn test_construct_subtree_with_new_leaf_create_extension() {
    //        root                           root
    //       /    \                         /    \
    //      o      o                       o      o
    //     / \                            / \
    //    o   existing_key    =>         o   subtree
    //                                      /       \
    //                                     y         placeholder
    //                                    / \
    //                                   x   placeholder
    //                                  / \
    //                      existing_key   new_key
    let existing_key = b"world".test_only_hash();
    let existing_blob = AccountStateBlob::from(b"world".to_vec());
    let existing_blob_hash = existing_blob.hash();
    let new_key = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob!!!!!".to_vec());
    assert_eq!(existing_key[0], 0b0100_0010);
    assert_eq!(new_key[0], 0b0100_1011);

    let existing_leaf = LeafNode::new(existing_key, LeafValue::BlobHash(existing_blob.hash()));

    let subtree = SparseMerkleTree::construct_subtree_with_new_leaf(
        new_key,
        new_blob.clone(),
        &existing_leaf,
        /* distance_from_root_to_existing_leaf = */ 2,
    );
    let smt = SparseMerkleTree { root: subtree };

    let new_blob_hash = new_blob.hash();
    let existing_leaf_hash = hash_leaf(existing_key, existing_blob_hash);
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    let x_hash = hash_internal(existing_leaf_hash, new_leaf_hash);
    let y_hash = hash_internal(x_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let root_hash = hash_internal(y_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
#[should_panic(expected = "Reached an internal node at the bottom of the tree.")]
fn test_construct_subtree_at_bottom_found_internal_node() {
    let left_child = Arc::new(SparseMerkleNode::new_subtree(HashValue::new(
        [1; HashValue::LENGTH],
    )));
    let right_child = Arc::new(SparseMerkleNode::new_empty());
    let current_node = Arc::new(SparseMerkleNode::new_internal(left_child, right_child));
    let key = b"hello".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob".to_vec());
    let remaining_bits = key.iter_bits();
    let proof_reader = ProofReader::default();
    let _subtree_node = SparseMerkleTree::construct_subtree_at_bottom(
        current_node,
        key,
        new_blob,
        remaining_bits,
        &proof_reader,
    );
}

#[test]
fn test_construct_subtree_at_bottom_found_leaf_node() {
    //        root                           root
    //       /    \                         /    \
    //      o      o                       o      o
    //     / \                            / \
    //    o   existing_key    =>         o   subtree
    //                                      /       \
    //                                     y         placeholder
    //                                    / \
    //                                   x   placeholder
    //                                  / \
    //                      existing_key   new_key
    let existing_key = b"world".test_only_hash();
    let existing_blob = AccountStateBlob::from(b"world".to_vec());
    let existing_blob_hash = existing_blob.hash();
    let new_key = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob!!!!!".to_vec());
    assert_eq!(existing_key[0], 0b0100_0010);
    assert_eq!(new_key[0], 0b0100_1011);

    let current_node = Arc::new(SparseMerkleNode::new_leaf(
        existing_key,
        LeafValue::BlobHash(existing_blob_hash),
    ));
    let remaining_bits = {
        let mut iter = new_key.iter_bits();
        iter.next();
        iter.next();
        iter
    };
    let leaf = Some((existing_key, existing_blob_hash));
    let siblings: Vec<_> = (0..2)
        .map(|x| HashValue::new([x; HashValue::LENGTH]))
        .collect();
    let proof = SparseMerkleProof::new(leaf, siblings);
    let proof_reader = ProofReader::new(vec![(new_key, proof)]);

    let subtree = SparseMerkleTree::construct_subtree_at_bottom(
        current_node,
        new_key,
        new_blob.clone(),
        remaining_bits,
        &proof_reader,
    )
    .unwrap();
    let smt = SparseMerkleTree { root: subtree };

    let existing_leaf_hash = hash_leaf(existing_key, existing_blob_hash);
    let new_blob_hash = new_blob.hash();
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    let x_hash = hash_internal(existing_leaf_hash, new_leaf_hash);
    let y_hash = hash_internal(x_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let root_hash = hash_internal(y_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
fn test_construct_subtree_at_bottom_found_empty_node() {
    //        root                  root
    //       /    \                /    \
    //      o      o              o      o
    //     / \                   / \
    //    o   placeholder    =>     o   new_key
    let new_key = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob!!!!!".to_vec());
    assert_eq!(new_key[0], 0b0100_1011);

    let current_node = Arc::new(SparseMerkleNode::new_empty());
    let remaining_bits = {
        let mut iter = new_key.iter_bits();
        // Skip first two.
        iter.next();
        iter.next();
        iter
    };
    let proof_reader = ProofReader::default();

    let subtree = SparseMerkleTree::construct_subtree_at_bottom(
        current_node,
        new_key,
        new_blob.clone(),
        remaining_bits,
        &proof_reader,
    )
    .unwrap();
    let smt = SparseMerkleTree { root: subtree };

    let new_blob_hash = new_blob.hash();
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    assert_eq!(smt.root_hash(), new_leaf_hash);
}

#[test]
fn test_construct_subtree_at_bottom_found_subtree_node() {
    //            root                  root
    //           /    \                /    \
    //          o      o              o      o
    //         / \                   / \
    //        o   subtree    =>     o   new_subtree
    //                                 /           \
    //                                x             sibling [5; 32] (from proof)
    //                               / \
    //   sibling [6; 32] (from proof)   new_leaf
    let new_key = b"aaaaaaaa".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob!!!!!".to_vec());
    assert_eq!(new_key[0], 0b0101_1111);

    let current_node = Arc::new(SparseMerkleNode::new_subtree(HashValue::new(
        [1; HashValue::LENGTH],
    )));
    let remaining_bits = {
        let mut iter = new_key.iter_bits();
        // Skip first two.
        iter.next();
        iter.next();
        iter
    };
    let leaf = None;
    let siblings: Vec<_> = (3..7)
        .rev()
        .map(|x| HashValue::new([x; HashValue::LENGTH]))
        .collect();
    let proof = SparseMerkleProof::new(leaf, siblings);
    let proof_reader = ProofReader::new(vec![(new_key, proof)]);

    let new_subtree = SparseMerkleTree::construct_subtree_at_bottom(
        current_node,
        new_key,
        new_blob.clone(),
        remaining_bits,
        &proof_reader,
    )
    .unwrap();
    let smt = SparseMerkleTree { root: new_subtree };

    let new_blob_hash = new_blob.hash();
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    let x_hash = hash_internal(HashValue::new([6; HashValue::LENGTH]), new_leaf_hash);
    let new_subtree_hash = hash_internal(x_hash, HashValue::new([5; HashValue::LENGTH]));
    assert_eq!(smt.root_hash(), new_subtree_hash);
}

#[test]
fn test_update_256_siblings_in_proof() {
    //                   root
    //                  /    \
    //                 o      placeholder
    //                / \
    //               o   placeholder
    //              / \
    //             .   placeholder
    //             .
    //             . (256 levels)
    //             o
    //            / \
    //        key1   key2
    let key1 = HashValue::new([0; HashValue::LENGTH]);
    let key2 = {
        let mut buf = key1.to_vec();
        *buf.last_mut().unwrap() |= 1;
        HashValue::from_slice(&buf).unwrap()
    };

    let blob1 = AccountStateBlob::from(b"value1".to_vec());
    let blob2 = AccountStateBlob::from(b"value2".to_vec());
    let value1_hash = blob1.hash();
    let value2_hash = blob2.hash();
    let leaf1_hash = hash_leaf(key1, value1_hash);
    let leaf2_hash = hash_leaf(key2, value2_hash);

    let mut siblings: Vec<_> = std::iter::repeat(*SPARSE_MERKLE_PLACEHOLDER_HASH)
        .take(255)
        .collect();
    siblings.push(leaf2_hash);
    siblings.reverse();
    let proof_of_key1 = SparseMerkleProof::new(Some((key1, value1_hash)), siblings.clone());

    let old_root_hash = siblings.iter().fold(leaf1_hash, |previous_hash, hash| {
        hash_internal(previous_hash, *hash)
    });
    assert!(proof_of_key1
        .verify(old_root_hash, key1, Some(&blob1))
        .is_ok());

    let new_blob1 = AccountStateBlob::from(b"value1111111111111".to_vec());
    let proof_reader = ProofReader::new(vec![(key1, proof_of_key1)]);
    let smt = SparseMerkleTree::new(old_root_hash);
    let new_smt = smt
        .update(vec![(key1, new_blob1.clone())], &proof_reader)
        .unwrap();

    let new_blob1_hash = new_blob1.hash();
    let new_leaf1_hash = hash_leaf(key1, new_blob1_hash);
    let new_root_hash = siblings.iter().fold(new_leaf1_hash, |previous_hash, hash| {
        hash_internal(previous_hash, *hash)
    });
    assert_eq!(new_smt.root_hash(), new_root_hash);

    assert_eq!(
        new_smt.get(key1),
        AccountState::ExistsInScratchPad(new_blob1)
    );
    assert_eq!(new_smt.get(key2), AccountState::Unknown);
}

#[test]
fn test_new_subtree() {
    let root_hash = HashValue::new([1; HashValue::LENGTH]);
    let smt = SparseMerkleTree::new(root_hash);
    assert!(smt.root.read_lock().is_subtree());
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
fn test_new_empty() {
    let root_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
    let smt = SparseMerkleTree::new(root_hash);
    assert!(smt.root.read_lock().is_empty());
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
fn test_update() {
    // Before the update, the tree was:
    //             root
    //            /    \
    //           y      key3
    //          / \
    //         x   placeholder
    //        / \
    //    key1   key2
    let key1 = b"aaaaa".test_only_hash();
    let key2 = b"bb".test_only_hash();
    let key3 = b"cccc".test_only_hash();
    assert_eq!(key1[0], 0b0000_0100);
    assert_eq!(key2[0], 0b0010_0100);
    assert_eq!(key3[0], 0b1110_0111);
    let value1 = AccountStateBlob::from(b"value1".to_vec());
    let value1_hash = value1.hash();
    let value2_hash = AccountStateBlob::from(b"value2".to_vec()).hash();
    let value3_hash = AccountStateBlob::from(b"value3".to_vec()).hash();

    // A new key at the "placeholder" position.
    let key4 = b"d".test_only_hash();
    assert_eq!(key4[0], 0b0100_1100);
    let value4 = AccountStateBlob::from(b"value".to_vec());

    // Create a proof for this new key.
    let leaf1_hash = hash_leaf(key1, value1_hash);
    let leaf2_hash = hash_leaf(key2, value2_hash);
    let leaf3_hash = hash_leaf(key3, value3_hash);
    let x_hash = hash_internal(leaf1_hash, leaf2_hash);
    let y_hash = hash_internal(x_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let old_root_hash = hash_internal(y_hash, leaf3_hash);
    let proof = SparseMerkleProof::new(None, vec![x_hash, leaf3_hash]);
    assert!(proof.verify(old_root_hash, key4, None).is_ok());

    // Create the old tree and update the tree with new value and proof.
    let proof_reader = ProofReader::new(vec![(key4, proof)]);
    let old_smt = SparseMerkleTree::new(old_root_hash);
    let smt1 = old_smt
        .update(vec![(key4, value4.clone())], &proof_reader)
        .unwrap();

    // Now smt1 should look like this:
    //             root
    //            /    \
    //           y      key3 (subtree)
    //          / \
    //         x   key4
    assert_eq!(smt1.get(key1), AccountState::Unknown);
    assert_eq!(smt1.get(key2), AccountState::Unknown);
    assert_eq!(smt1.get(key3), AccountState::Unknown);
    assert_eq!(
        smt1.get(key4),
        AccountState::ExistsInScratchPad(value4.clone())
    );

    let non_existing_key = b"foo".test_only_hash();
    assert_eq!(non_existing_key[0], 0b0111_0110);
    assert_eq!(smt1.get(non_existing_key), AccountState::DoesNotExist);

    // Verify root hash.
    let value4_hash = value4.hash();
    let leaf4_hash = hash_leaf(key4, value4_hash);
    let y_hash = hash_internal(x_hash, leaf4_hash);
    let root_hash = hash_internal(y_hash, leaf3_hash);
    assert_eq!(smt1.root_hash(), root_hash);

    // Next, we are going to modify key1. Create a proof for key1.
    let proof = SparseMerkleProof::new(
        Some((key1, value1_hash)),
        vec![leaf2_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH, leaf3_hash],
    );
    assert!(proof.verify(old_root_hash, key1, Some(&value1)).is_ok());

    let value1 = AccountStateBlob::from(b"value11111".to_vec());
    let proof_reader = ProofReader::new(vec![(key1, proof)]);
    let smt2 = smt1
        .update(vec![(key1, value1.clone())], &proof_reader)
        .unwrap();

    // Now the tree looks like:
    //              root
    //             /    \
    //            y      key3 (subtree)
    //           / \
    //          x   key4
    //         / \
    //     key1    key2 (subtree)
    assert_eq!(
        smt2.get(key1),
        AccountState::ExistsInScratchPad(value1.clone())
    );
    assert_eq!(smt2.get(key2), AccountState::Unknown);
    assert_eq!(smt2.get(key3), AccountState::Unknown);
    assert_eq!(smt2.get(key4), AccountState::ExistsInScratchPad(value4));

    // Verify root hash.
    let value1_hash = value1.hash();
    let leaf1_hash = hash_leaf(key1, value1_hash);
    let x_hash = hash_internal(leaf1_hash, leaf2_hash);
    let y_hash = hash_internal(x_hash, leaf4_hash);
    let root_hash = hash_internal(y_hash, leaf3_hash);
    assert_eq!(smt2.root_hash(), root_hash);

    // We now try to create another branch on top of smt1.
    let value4 = AccountStateBlob::from(b"new value 4444444444".to_vec());
    // key4 already exists in the tree.
    let proof_reader = ProofReader::default();
    let smt22 = smt1
        .update(vec![(key4, value4.clone())], &proof_reader)
        .unwrap();

    assert_eq!(smt22.get(key1), AccountState::Unknown);
    assert_eq!(smt22.get(key2), AccountState::Unknown);
    assert_eq!(smt22.get(key3), AccountState::Unknown);
    assert_eq!(
        smt22.get(key4),
        AccountState::ExistsInScratchPad(value4.clone())
    );

    // Now prune smt1.
    smt1.prune();

    // For smt2, only key1 should be available since smt2 was constructed by updating smt1 with
    // key1.
    assert_eq!(smt2.get(key1), AccountState::ExistsInScratchPad(value1));
    assert_eq!(smt2.get(key2), AccountState::Unknown);
    assert_eq!(smt2.get(key3), AccountState::Unknown);
    assert_eq!(smt2.get(key4), AccountState::Unknown);

    // For smt22, only key4 should be available since smt22 was constructed by updating smt1 with
    // key4.
    assert_eq!(smt22.get(key1), AccountState::Unknown);
    assert_eq!(smt22.get(key2), AccountState::Unknown);
    assert_eq!(smt22.get(key3), AccountState::Unknown);
    assert_eq!(smt22.get(key4), AccountState::ExistsInScratchPad(value4));
}
