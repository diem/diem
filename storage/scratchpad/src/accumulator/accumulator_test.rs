// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Accumulator;
use crypto::{
    hash::{CryptoHash, TestOnlyHash, TestOnlyHasher, ACCUMULATOR_PLACEHOLDER_HASH},
    HashValue,
};
use types::proof::TestAccumulatorInternalNode;

// Computes the root hash of an accumulator with given leaves.
fn compute_root_hash_naive(leaves: &[HashValue]) -> HashValue {
    if leaves.is_empty() {
        return *ACCUMULATOR_PLACEHOLDER_HASH;
    }

    let mut current_level = leaves.to_vec();
    current_level.resize(
        leaves.len().next_power_of_two(),
        *ACCUMULATOR_PLACEHOLDER_HASH,
    );

    while current_level.len() > 1 {
        assert!(current_level.len().is_power_of_two());

        let mut parent_level = vec![];
        for (index, hash) in current_level.iter().enumerate().step_by(2) {
            let left_hash = hash;
            let right_hash = &current_level[index + 1];
            let parent_hash = if *left_hash == *ACCUMULATOR_PLACEHOLDER_HASH
                && *right_hash == *ACCUMULATOR_PLACEHOLDER_HASH
            {
                *ACCUMULATOR_PLACEHOLDER_HASH
            } else {
                TestAccumulatorInternalNode::new(*left_hash, *right_hash).hash()
            };
            parent_level.push(parent_hash);
        }

        current_level = parent_level;
    }

    assert_eq!(current_level.len(), 1);
    current_level.remove(0)
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
    let mut accumulator = Accumulator::<TestOnlyHasher>::default();
    // Append the leaves one at a time and check the root hashes match.
    for (i, (leaf, expected_root_hash)) in
        itertools::zip_eq(leaves.into_iter(), expected_root_hashes.into_iter()).enumerate()
    {
        assert_eq!(accumulator.root_hash(), expected_root_hash);
        assert_eq!(accumulator.num_leaves(), i as u64);
        accumulator = accumulator.append(vec![leaf]);
    }
}
