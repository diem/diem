// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use diem_crypto::{
    hash::{CryptoHash, TestOnlyHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_temppath::TempPath;
use diem_types::{
    account_address::HashAccountAddress,
    account_state_blob::AccountStateBlob,
    proof::{SparseMerkleLeafNode, SparseMerkleProof},
};
use diemdb::{test_helper::arb_blocks_to_commit, DiemDB};
use proptest::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;
use storage_interface::{DbReader, DbWriter};

fn update_byte(original_key: &HashValue, n: usize, byte: u8) -> HashValue {
    let mut key = original_key.to_vec();
    key[n] = byte;
    HashValue::from_slice(&key).unwrap()
}

fn hash_internal(left_child: HashValue, right_child: HashValue) -> HashValue {
    diem_types::proof::SparseMerkleInternalNode::new(left_child, right_child).hash()
}

fn hash_leaf(key: HashValue, value_hash: HashValue) -> HashValue {
    SparseMerkleLeafNode::new(key, value_hash).hash()
}

struct ProofReader<V>(HashMap<HashValue, SparseMerkleProof<V>>);

impl<V: Sync> ProofReader<V> {
    fn new(key_with_proof: Vec<(HashValue, SparseMerkleProof<V>)>) -> Self {
        ProofReader(key_with_proof.into_iter().collect())
    }
}

impl<V: Sync> Default for ProofReader<V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<V: Sync> ProofRead<V> for ProofReader<V> {
    fn get_proof(&self, key: HashValue) -> Option<&SparseMerkleProof<V>> {
        self.0.get(&key)
    }
}

type SparseMerkleTree = super::SparseMerkleTree<AccountStateBlob>;
type SubTree = super::SubTree<AccountStateBlob>;

#[test]
fn test_construct_subtree_zero_siblings() {
    let node_hash = HashValue::new([1; HashValue::LENGTH]);
    let leaf = SubTree::new_unknown(node_hash);
    let subtree = SparseMerkleTree::construct_subtree(std::iter::empty(), std::iter::empty(), leaf);
    assert_eq!(subtree.hash(), node_hash);
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

    let leaf = SubTree::new_leaf_with_value_hash(key, blob.hash());
    let bits = vec![false, false, true];
    let a_hash = HashValue::new([2; HashValue::LENGTH]);
    let b_hash = HashValue::new([3; HashValue::LENGTH]);
    let c_hash = HashValue::new([4; HashValue::LENGTH]);
    let siblings = vec![a_hash, b_hash, c_hash]
        .into_iter()
        .map(SubTree::new_unknown);
    let subtree = SparseMerkleTree::construct_subtree(bits.into_iter(), siblings, leaf);

    let z_hash = hash_internal(leaf_hash, a_hash);
    let y_hash = hash_internal(z_hash, b_hash);
    let root_hash = hash_internal(c_hash, y_hash);
    assert_eq!(subtree.hash(), root_hash);
}

#[test]
#[should_panic]
fn test_construct_subtree_panic() {
    let node_hash = HashValue::new([1; HashValue::LENGTH]);
    let unknown = SubTree::new_unknown(node_hash);
    let _ = SparseMerkleTree::construct_subtree(std::iter::once(true), std::iter::empty(), unknown);
}

#[test]
fn test_construct_subtree_with_new_leaf_override_existing_leaf() {
    let key = b"hello".test_only_hash();
    let old_blob = AccountStateBlob::from(b"old_old_old".to_vec());
    let new_blob = AccountStateBlob::from(b"new_new_new".to_vec());

    let existing_leaf = SubTree::new_leaf_with_value_hash(key, old_blob.hash());

    let subtree = SparseMerkleTree::construct_subtree_with_new_leaf(
        key,
        new_blob.clone(),
        existing_leaf.clone(),
        key,
        /* distance_from_root_to_existing_leaf = */ 3,
    );

    let new_blob_hash = new_blob.hash();
    let root_hash = hash_leaf(key, new_blob_hash);
    assert_eq!(subtree.hash(), root_hash);

    // batch version api
    let subtree_batch = SparseMerkleTree::batch_construct_subtree_with_existing_leaf(
        vec![(key, &new_blob)].as_slice(),
        existing_leaf,
        key,
        /* depth = */ 3,
    );

    assert_eq!(subtree_batch.hash(), root_hash);
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

    let existing_leaf = SubTree::new_leaf_with_value_hash(existing_key, existing_blob.hash());

    let new_blob_hash = new_blob.hash();
    let existing_leaf_hash = hash_leaf(existing_key, existing_blob_hash);
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    let x_hash = hash_internal(existing_leaf_hash, new_leaf_hash);
    let y_hash = hash_internal(x_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let root_hash = hash_internal(y_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);

    let subtree = SparseMerkleTree::construct_subtree_with_new_leaf(
        new_key,
        new_blob.clone(),
        existing_leaf.clone(),
        existing_key,
        /* distance_from_root_to_existing_leaf = */ 2,
    );
    assert_eq!(subtree.hash(), root_hash);

    // batch version api
    let subtree_batch = SparseMerkleTree::batch_construct_subtree_with_existing_leaf(
        vec![(new_key, &new_blob)].as_slice(),
        existing_leaf,
        existing_key,
        /* depth = */ 2,
    );
    assert_eq!(subtree_batch.hash(), root_hash);
}

#[test]
#[should_panic(expected = "Reached an internal node at the bottom of the tree.")]
fn test_construct_subtree_at_bottom_found_internal_node() {
    let left_child = SubTree::new_unknown(HashValue::new([1; HashValue::LENGTH]));
    let right_child = SubTree::new_empty();
    let current = SubTree::new_internal(left_child, right_child);

    let key = b"hello".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob".to_vec());
    let remaining_bits = key.iter_bits();
    let proof_reader = ProofReader::default();
    let _ = SparseMerkleTree::construct_subtree_at_bottom(
        &current,
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

    let current = SubTree::new_leaf_with_value_hash(existing_key, existing_blob_hash);
    let remaining_bits = {
        let mut iter = new_key.iter_bits();
        iter.next();
        iter.next();
        iter
    };
    let leaf = Some(SparseMerkleLeafNode::new(existing_key, existing_blob_hash));
    let siblings: Vec<_> = (0..2)
        .map(|x| HashValue::new([x; HashValue::LENGTH]))
        .collect();
    let proof = SparseMerkleProof::new(leaf, siblings);
    let proof_reader = ProofReader::new(vec![(new_key, proof)]);

    let existing_leaf_hash = hash_leaf(existing_key, existing_blob_hash);
    let new_blob_hash = new_blob.hash();
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    let x_hash = hash_internal(existing_leaf_hash, new_leaf_hash);
    let y_hash = hash_internal(x_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let root_hash = hash_internal(y_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);

    let subtree = SparseMerkleTree::construct_subtree_at_bottom(
        &current,
        new_key,
        new_blob,
        remaining_bits,
        &proof_reader,
    )
    .unwrap();
    assert_eq!(subtree.hash(), root_hash);
}

#[test]
fn test_construct_subtree_at_bottom_found_empty_node() {
    //        root                      root
    //       /    \                    /    \
    //      o      o                  o      o
    //     / \                       / \
    //    o   placeholder    =>     o   new_key
    let new_key = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".test_only_hash();
    let new_blob = AccountStateBlob::from(b"new_blob!!!!!".to_vec());
    assert_eq!(new_key[0], 0b0100_1011);

    let current = SubTree::new_empty();
    let remaining_bits = {
        let mut iter = new_key.iter_bits();
        // Skip first two.
        iter.next();
        iter.next();
        iter
    };
    let proof_reader = ProofReader::default();

    let new_blob_hash = new_blob.hash();
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);

    let subtree = SparseMerkleTree::construct_subtree_at_bottom(
        &current,
        new_key,
        new_blob,
        remaining_bits,
        &proof_reader,
    )
    .unwrap();
    assert_eq!(subtree.hash(), new_leaf_hash);
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

    let current = SubTree::new_unknown(HashValue::new([1; HashValue::LENGTH]));
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

    let new_blob_hash = new_blob.hash();
    let new_leaf_hash = hash_leaf(new_key, new_blob_hash);
    let x_hash = hash_internal(HashValue::new([6; HashValue::LENGTH]), new_leaf_hash);
    let new_subtree_hash = hash_internal(x_hash, HashValue::new([5; HashValue::LENGTH]));

    let subtree = SparseMerkleTree::construct_subtree_at_bottom(
        &current,
        new_key,
        new_blob,
        remaining_bits,
        &proof_reader,
    )
    .unwrap();
    assert_eq!(subtree.hash(), new_subtree_hash);
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
    let proof_of_key1 = SparseMerkleProof::new(
        Some(SparseMerkleLeafNode::new(key1, value1_hash)),
        siblings.clone(),
    );

    let old_root_hash = siblings.iter().fold(leaf1_hash, |previous_hash, hash| {
        hash_internal(previous_hash, *hash)
    });
    assert!(proof_of_key1
        .verify(old_root_hash, key1, Some(&blob1))
        .is_ok());

    let new_blob1 = AccountStateBlob::from(b"value1111111111111".to_vec());
    let proof_reader = ProofReader::new(vec![(key1, proof_of_key1)]);
    let smt = SparseMerkleTree::new(old_root_hash);
    let new_smt = smt.update(vec![(key1, &new_blob1)], &proof_reader).unwrap();

    let new_blob1_hash = new_blob1.hash();
    let new_leaf1_hash = hash_leaf(key1, new_blob1_hash);
    let new_root_hash = siblings.iter().fold(new_leaf1_hash, |previous_hash, hash| {
        hash_internal(previous_hash, *hash)
    });
    assert_eq!(new_smt.root_hash(), new_root_hash);

    assert_eq!(
        new_smt.get(key1),
        AccountStatus::ExistsInScratchPad(new_blob1)
    );
    assert_eq!(new_smt.get(key2), AccountStatus::Unknown);
}

#[test]
fn test_new_subtree() {
    let root_hash = HashValue::new([1; HashValue::LENGTH]);
    let smt = SparseMerkleTree::new(root_hash);
    assert!(smt.root_weak().is_unknown());
    assert_eq!(smt.root_hash(), root_hash);
}

#[test]
fn test_new_empty() {
    let root_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
    let smt = SparseMerkleTree::new(root_hash);
    assert!(smt.root_weak().is_empty());
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
    let leaf1 = SparseMerkleLeafNode::new(key1, value1_hash);
    let leaf1_hash = leaf1.hash();
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
        .update(vec![(key4, &value4)], &proof_reader)
        .unwrap();

    // Now smt1 should look like this:
    //             root
    //            /    \
    //           y      key3 (subtree)
    //          / \
    //         x   key4
    assert_eq!(smt1.get(key1), AccountStatus::Unknown);
    assert_eq!(smt1.get(key2), AccountStatus::Unknown);
    assert_eq!(smt1.get(key3), AccountStatus::Unknown);
    assert_eq!(
        smt1.get(key4),
        AccountStatus::ExistsInScratchPad(value4.clone())
    );

    let non_existing_key = b"foo".test_only_hash();
    assert_eq!(non_existing_key[0], 0b0111_0110);
    assert_eq!(smt1.get(non_existing_key), AccountStatus::DoesNotExist);

    // Verify root hash.
    let value4_hash = value4.hash();
    let leaf4_hash = hash_leaf(key4, value4_hash);
    let y_hash = hash_internal(x_hash, leaf4_hash);
    let root_hash = hash_internal(y_hash, leaf3_hash);
    assert_eq!(smt1.root_hash(), root_hash);

    // Next, we are going to modify key1. Create a proof for key1.
    let proof = SparseMerkleProof::new(
        Some(leaf1),
        vec![leaf2_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH, leaf3_hash],
    );
    assert!(proof.verify(old_root_hash, key1, Some(&value1)).is_ok());

    let value1 = AccountStateBlob::from(b"value11111".to_vec());
    let proof_reader = ProofReader::new(vec![(key1, proof)]);
    let smt2 = smt1.update(vec![(key1, &value1)], &proof_reader).unwrap();

    // Now the tree looks like:
    //              root
    //             /    \
    //            y      key3 (subtree)
    //           / \
    //          x   key4 (from smt1)
    //         / \
    //     key1    key2 (subtree)
    assert_eq!(
        smt2.get(key1),
        AccountStatus::ExistsInScratchPad(value1.clone())
    );
    assert_eq!(smt2.get(key2), AccountStatus::Unknown);
    assert_eq!(smt2.get(key3), AccountStatus::Unknown);
    assert_eq!(smt2.get(key4), AccountStatus::ExistsInScratchPad(value4));

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
    let smt22 = smt1.update(vec![(key4, &value4)], &proof_reader).unwrap();

    // smt22 is like:
    // Now smt1 should look like this:
    //             root
    //            /    \
    //           y'      key3 (subtree)
    //          / \
    //         x   key4
    assert_eq!(smt22.get(key1), AccountStatus::Unknown);
    assert_eq!(smt22.get(key2), AccountStatus::Unknown);
    assert_eq!(smt22.get(key3), AccountStatus::Unknown);
    assert_eq!(
        smt22.get(key4),
        AccountStatus::ExistsInScratchPad(value4.clone())
    );

    // Now prune smt1.
    smt1.prune();

    // For smt2, only key1 should be available since smt2 was constructed by updating smt1 with
    // key1.
    assert_eq!(smt2.get(key1), AccountStatus::ExistsInScratchPad(value1));
    assert_eq!(smt2.get(key2), AccountStatus::Unknown);
    assert_eq!(smt2.get(key3), AccountStatus::Unknown);
    assert_eq!(smt2.get(key4), AccountStatus::Unknown);

    // For smt22, only key4 should be available since smt22 was constructed by updating smt1 with
    // key4.
    assert_eq!(smt22.get(key1), AccountStatus::Unknown);
    assert_eq!(smt22.get(key2), AccountStatus::Unknown);
    assert_eq!(smt22.get(key3), AccountStatus::Unknown);
    assert_eq!(smt22.get(key4), AccountStatus::ExistsInScratchPad(value4));
}

#[test]
fn test_drop() {
    let mut smt = SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH);
    let proof_reader = ProofReader::default();
    for _ in 0..100000 {
        smt = smt
            .update(
                vec![(HashValue::zero(), &AccountStateBlob::from(b"zero".to_vec()))],
                &proof_reader,
            )
            .unwrap()
    }

    // smt with a lot of ancestors being dropped here. It's a stack overflow if a manual iterative
    // `Drop` implementation is not in place.
}

#[test]
fn test_batch_update_construct_subtree_from_proofs() {
    // Before the update, the tree was:
    //             root
    //            /    \
    //           y      key3
    //          / \
    //         x   placeholder
    //        / \
    //    key1   key2
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = AccountStateBlob::from(b"value1".to_vec());

    let key2 = update_byte(&key1, 0, 0b0010_0100);
    let value2 = AccountStateBlob::from(b"value2".to_vec());

    let key3 = update_byte(&key1, 0, 0b1110_0111);
    let value3 = AccountStateBlob::from(b"value3".to_vec());
    let new_value3 = AccountStateBlob::from(b"new_value3".to_vec());

    let value1_hash = value1.hash();
    let value2_hash = value2.hash();
    let value3_hash = value3.hash();

    // A new key at the "placeholder" position.
    let key4 = update_byte(&key1, 0, 0b0100_1100);
    let value4 = AccountStateBlob::from(b"value4".to_vec());

    // A new key that shares path with key1.
    let key5 = update_byte(&key1, 0, 0b0001_0000);
    let value5 = AccountStateBlob::from(b"value5".to_vec());

    // Two new keys that shares path with key2.
    let key6 = update_byte(&key1, 0, 0b0010_0000);
    let value6 = AccountStateBlob::from(b"value6".to_vec());
    let key7 = update_byte(&key1, 0, 0b0011_0000);
    let value7 = AccountStateBlob::from(b"value7".to_vec());

    // Create a proof for this new key.
    let leaf1 = SparseMerkleLeafNode::new(key1, value1_hash);
    let leaf2 = SparseMerkleLeafNode::new(key2, value2_hash);
    let leaf3 = SparseMerkleLeafNode::new(key3, value3_hash);
    let leaf1_hash = leaf1.hash();
    let leaf2_hash = leaf2.hash();
    let leaf3_hash = leaf3.hash();
    let x_hash = hash_internal(leaf1_hash, leaf2_hash);
    let y_hash = hash_internal(x_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let old_root_hash = hash_internal(y_hash, leaf3_hash);

    let proof3 = SparseMerkleProof::new(Some(leaf3), vec![y_hash]);
    assert!(proof3.verify(old_root_hash, key3, Some(&value3)).is_ok());
    let proof4 = SparseMerkleProof::new(None, vec![x_hash, leaf3_hash]);
    assert!(proof4.verify(old_root_hash, key4, None).is_ok());
    let proof5 = SparseMerkleProof::new(
        Some(leaf1),
        vec![leaf2_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH, leaf3_hash],
    );
    assert!(proof5.verify(old_root_hash, key5, None).is_ok());

    let proof67 = SparseMerkleProof::new(
        Some(leaf2),
        vec![leaf1_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH, leaf3_hash],
    );
    assert!(proof67.verify(old_root_hash, key6, None).is_ok());
    assert!(proof67.verify(old_root_hash, key7, None).is_ok());

    // Create the old tree and update the tree with new value and proof.
    let proof_reader = ProofReader::new(vec![
        (key3, proof3),
        (key4, proof4),
        (key5, proof5),
        (key6, proof67.clone()),
        (key7, proof67),
    ]);

    let smt_serial = SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH)
        .serial_update(
            vec![vec![
                (key3, &new_value3),
                (key4, &value4),
                (key5, &value5),
                (key6, &value6),
                (key7, &value7),
            ]],
            &proof_reader,
        )
        .unwrap()
        .1;

    let smt_batch = SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH)
        .batch_update(
            vec![
                (key3, &new_value3),
                (key4, &value4),
                (key5, &value5),
                (key6, &value6),
                (key7, &value7),
            ],
            &proof_reader,
        )
        .unwrap();

    assert_eq!(smt_serial.root_hash(), smt_batch.root_hash());
}

#[test]
fn test_update_consistency() {
    let seed: &[_] = &[1, 2, 3, 4];
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(&seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);

    let mut kvs = vec![];
    let smt = SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH);
    let proof_reader = ProofReader::default();
    let mut smt_serial;
    let mut smt_batch;
    for i in 1..=5 {
        for _ in 0..1000 {
            let key = HashValue::random_with_rng(&mut rng);
            let value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
            kvs.push((key, value));
        }
        let batch = kvs.iter().map(|(k, v)| (*k, v)).collect::<Vec<_>>();
        let start = std::time::Instant::now();
        smt_serial = smt.update(batch.clone(), &proof_reader).unwrap();
        println!("serial {}-th run: {}ms", i, start.elapsed().as_millis());

        let start = std::time::Instant::now();
        smt_batch = smt.batch_update(batch.clone(), &proof_reader).unwrap();
        println!("batch {}-th run: {}ms", i, start.elapsed().as_millis());
        kvs.clear();
        assert_eq!(smt_serial.root_hash(), smt_batch.root_hash());
    }
}

#[test]
fn test_update_consistency_batches() {
    let seed: &[_] = &[1, 2, 3, 4];
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(&seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);

    let mut kvs = vec![];
    let smt = SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH);
    let proof_reader = ProofReader::default();
    let mut smt_serial;
    let mut smt_batches;
    for i in 1..=5 {
        for _ in 0..1000 {
            let key = HashValue::random_with_rng(&mut rng);
            let value = AccountStateBlob::from(HashValue::random_with_rng(&mut rng).to_vec());
            kvs.push((key, value));
        }

        let mut batches = vec![];
        for j in 0..500 {
            batches.push(
                kvs.iter()
                    .skip(j * 2)
                    .take(2)
                    .map(|(k, v)| (*k, v))
                    .collect::<Vec<_>>(),
            );
        }

        let start = std::time::Instant::now();
        smt_serial = smt.serial_update(batches.clone(), &proof_reader).unwrap();
        println!("serial {}-th run: {}ms", i, start.elapsed().as_millis());

        let start = std::time::Instant::now();
        smt_batches = smt.batches_update(batches.clone(), &proof_reader).unwrap();
        println!("batches {}-th run: {}ms", i, start.elapsed().as_millis());

        assert_eq!(smt_serial.0, smt_batches.0);
        assert_eq!(smt_serial.1.root_hash(), smt_batches.1.root_hash());

        kvs.clear();
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn test_read_partial_tree_from_storage(input in arb_blocks_to_commit().no_shrink()) {
        let tmp_dir = TempPath::new();
        let db = DiemDB::new_for_test(&tmp_dir);
        // Insert a transaction to ensure a non-empty ledger;

        let tree_state = db.get_latest_tree_state().unwrap();
        let initial_state_root = tree_state.account_state_root_hash;
        let mut cur_ver = -1i64;

        prop_assert_eq!(initial_state_root, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        let mut serial_updated_tree = SparseMerkleTree::new(initial_state_root);
        let mut batch_updated_tree = SparseMerkleTree::new(initial_state_root);
        let mut batches_updated_tree = SparseMerkleTree::new(initial_state_root);

        for (txns_to_commit, ledger_info_with_sigs) in input.iter() {
            let updates = txns_to_commit.iter()
                .map(|txn_to_commit| txn_to_commit
                     .account_states()
                     .iter()
                     .map(|(a, b)|(a.hash(), b))
                     .collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let account_state_proofs = txns_to_commit.iter()
                .map(|txn_to_commit| txn_to_commit
                     .account_states()
                     .iter()
                     .collect::<Vec<_>>())
                .flatten()
                .map(|(k, _)| {
                    if cur_ver == -1 {
                        SparseMerkleProof::<AccountStateBlob>::new(None, vec![])
                    } else {
                        db.get_account_state_with_proof(*k, cur_ver as u64, cur_ver as u64)
                            .map(|p| p.proof.transaction_info_to_account_proof().clone())
                            .unwrap()
                    }
                });

            let proof_reader = ProofReader::new(
                itertools::zip_eq(
                    updates.iter().flatten().map(|(k, _)| *k),
                    account_state_proofs,
                ).collect::<Vec<_>>(),
            );

            db.save_transactions(
                &txns_to_commit,
                (cur_ver + 1) as u64, /* first_version */
                Some(ledger_info_with_sigs),
            )
                .unwrap();

            let tree_state = db.get_latest_tree_state().unwrap();
            let root_in_db = tree_state.account_state_root_hash;
            cur_ver = tree_state.num_transactions as i64 - 1;

            batch_updated_tree = batch_updated_tree.batch_update(updates
                                                                 .iter()
                                                                 .flatten()
                                                                 .map(|(k, v)| (*k, *v))
                                                                 .collect::<Vec<_>>(),
                                                                 &proof_reader)
                .unwrap();
            serial_updated_tree = serial_updated_tree
                .serial_update(updates.clone(), &proof_reader)
                .unwrap().1;
            batches_updated_tree = batches_updated_tree
                .batches_update(updates, &proof_reader)
                .unwrap().1;

            prop_assert_eq!(batch_updated_tree.root_hash(), batches_updated_tree.root_hash());
            prop_assert_eq!(batch_updated_tree.root_hash(), serial_updated_tree.root_hash());
            prop_assert_eq!(batch_updated_tree.root_hash(), root_in_db);
            batch_updated_tree.prune();
            serial_updated_tree.prune();
            batches_updated_tree.prune();
        }
    }
}
