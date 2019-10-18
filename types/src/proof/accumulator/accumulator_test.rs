// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::InMemoryAccumulator;
use crate::proof::{
    definition::LeafCount,
    position::{FrozenSubtreeSiblingIterator, Position},
    TestAccumulatorInternalNode,
};
use libra_crypto::{
    hash::{CryptoHash, TestOnlyHash, TestOnlyHasher, ACCUMULATOR_PLACEHOLDER_HASH},
    HashValue,
};
use proptest::{collection::vec, prelude::*};
use std::collections::HashMap;

fn compute_parent_hash(left_hash: HashValue, right_hash: HashValue) -> HashValue {
    if left_hash == *ACCUMULATOR_PLACEHOLDER_HASH && right_hash == *ACCUMULATOR_PLACEHOLDER_HASH {
        *ACCUMULATOR_PLACEHOLDER_HASH
    } else {
        TestAccumulatorInternalNode::new(left_hash, right_hash).hash()
    }
}

/// Given a list of leaves, constructs the smallest accumulator that has all the leaves and
/// computes the hash of every node in the tree.
fn compute_hashes_for_all_positions(leaves: &[HashValue]) -> HashMap<Position, HashValue> {
    if leaves.is_empty() {
        return HashMap::new();
    }

    let mut current_leaves = leaves.to_vec();
    current_leaves.resize(
        leaves.len().next_power_of_two(),
        *ACCUMULATOR_PLACEHOLDER_HASH,
    );
    let mut position_to_hash = HashMap::new();
    let mut current_level = 0;

    while current_leaves.len() > 1 {
        assert!(current_leaves.len().is_power_of_two());

        let mut parent_leaves = vec![];
        for (index, _hash) in current_leaves.iter().enumerate().step_by(2) {
            let left_hash = current_leaves[index];
            let right_hash = current_leaves[index + 1];
            let parent_hash = compute_parent_hash(left_hash, right_hash);
            parent_leaves.push(parent_hash);

            let left_pos = Position::from_level_and_pos(current_level, index as u64);
            let right_pos = Position::from_level_and_pos(current_level, index as u64 + 1);
            assert_eq!(position_to_hash.insert(left_pos, left_hash), None);
            assert_eq!(position_to_hash.insert(right_pos, right_hash), None);
        }

        assert_eq!(current_leaves.len(), parent_leaves.len() << 1);
        current_leaves = parent_leaves;
        current_level += 1;
    }

    assert_eq!(
        position_to_hash.insert(
            Position::from_level_and_pos(current_level, 0),
            current_leaves[0],
        ),
        None,
    );
    position_to_hash
}

// Computes the root hash of an accumulator with given leaves.
fn compute_root_hash_naive(leaves: &[HashValue]) -> HashValue {
    let position_to_hash = compute_hashes_for_all_positions(leaves);
    if position_to_hash.is_empty() {
        return *ACCUMULATOR_PLACEHOLDER_HASH;
    }

    let rightmost_leaf_index = leaves.len() as u64 - 1;
    *position_to_hash
        .get(&Position::root_from_leaf_index(rightmost_leaf_index))
        .expect("Root position should exist in the map.")
}

// Helper function to create a list of leaves.
fn create_leaves(nums: std::ops::Range<usize>) -> Vec<HashValue> {
    nums.map(|x| x.to_be_bytes().test_only_hash()).collect()
}

#[test]
fn test_accumulator_append() {
    // expected_root_hashes[i] is the root hash of an accumulator that has the first i leaves.
    let expected_root_hashes: Vec<HashValue> = (0..100)
        .map(|x| {
            let leaves = create_leaves(0..x);
            compute_root_hash_naive(&leaves)
        })
        .collect();

    let leaves = create_leaves(0..100);
    let mut accumulator = InMemoryAccumulator::<TestOnlyHasher>::default();
    // Append the leaves one at a time and check the root hashes match.
    for (i, (leaf, expected_root_hash)) in
        itertools::zip_eq(leaves.into_iter(), expected_root_hashes.into_iter()).enumerate()
    {
        assert_eq!(accumulator.root_hash(), expected_root_hash);
        assert_eq!(accumulator.num_leaves(), i as LeafCount);
        accumulator = accumulator.append(&[leaf]);
    }
}

proptest! {
    #[test]
    fn test_accumulator_append_subtrees(
        hashes1 in vec(any::<HashValue>(), 0..100),
        hashes2 in vec(any::<HashValue>(), 0..100),
    ) {
        // Construct an accumulator with hashes1.
        let accumulator = InMemoryAccumulator::<TestOnlyHasher>::from_leaves(&hashes1);

        // Compute all the internal nodes in a bigger accumulator with combination of hashes1 and
        // hashes2.
        let mut all_hashes = hashes1.clone();
        all_hashes.extend_from_slice(&hashes2);
        let position_to_hash = compute_hashes_for_all_positions(&all_hashes);

        let subtree_hashes: Vec<_> =
            FrozenSubtreeSiblingIterator::new(hashes1.len() as LeafCount, all_hashes.len() as LeafCount)
                .filter_map(|pos| position_to_hash.get(&pos).cloned())
                .collect();
        let new_accumulator = accumulator
            .append_subtrees(&subtree_hashes, hashes2.len() as LeafCount)
            .unwrap();
        prop_assert_eq!(
            new_accumulator.root_hash(),
            compute_root_hash_naive(&all_hashes)
        );
    }
}
