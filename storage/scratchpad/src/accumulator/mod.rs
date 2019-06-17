// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements an in-memory Merkle Accumulator that is similar to what we use in
//! storage. This accumulator will only store a small portion of the tree -- for any subtree that
//! is full, we store only the root. Also we only store the frozen nodes, therefore this structure
//! will always store up to `Log(n)` number of nodes, where `n` is the total number of elements in
//! the tree.
//!
//! This accumulator is immutable once constructed. If we append new elements to the tree we will
//! obtain a new accumulator instance and the old one remains unchanged.

#[cfg(test)]
mod accumulator_test;

use crypto::{
    hash::{CryptoHash, CryptoHasher, ACCUMULATOR_PLACEHOLDER_HASH},
    HashValue,
};
use std::marker::PhantomData;
use types::proof::{position::Position, treebits::NodeDirection, MerkleTreeInternalNode};

/// The Accumulator implementation.
#[derive(Default)]
pub struct Accumulator<H> {
    /// Represents the roots of all the full subtrees from left to right in this accumulator. For
    /// example, if we have the following accumulator, this vector will have two hashes that
    /// correspond to `X` and `e`.
    /// ```text
    ///                 root
    ///                /    \
    ///              /        \
    ///            /            \
    ///           X              o
    ///         /   \           / \
    ///        /     \         /   \
    ///       o       o       o     placeholder
    ///      / \     / \     / \
    ///     a   b   c   d   e   placeholder
    /// ```
    frozen_subtree_roots: Vec<HashValue>,

    /// The total number of elements in this accumulator.
    num_elements: u64,

    phantom: PhantomData<H>,
}

impl<H> Accumulator<H>
where
    H: CryptoHasher,
{
    /// Constructs a new accumulator with roots of existing frozen subtrees. At the beginning this
    /// will be an empty vector and `num_elements` will be zero. Later if we restart and the
    /// storage have persisted some elements, we will load them from storage.
    pub fn new(frozen_subtree_roots: Vec<HashValue>, num_elements: u64) -> Self {
        assert_eq!(
            frozen_subtree_roots.len(),
            num_elements.count_ones() as usize,
            "The number of frozen subtrees does not match the number of elements. \
             frozen_subtree_roots.len(): {}. num_elements: {}.",
            frozen_subtree_roots.len(),
            num_elements,
        );

        Accumulator {
            frozen_subtree_roots,
            num_elements,
            phantom: PhantomData,
        }
    }

    /// Appends a list of new elements to an existing accumulator. Since the accumulator is
    /// immutable, the existing one remains unchanged and a new one representing the result is
    /// returned.
    pub fn append(&self, elements: Vec<HashValue>) -> Self {
        let mut frozen_subtree_roots = self.frozen_subtree_roots.clone();
        let mut num_elements = self.num_elements;
        for element in elements {
            Self::append_one(&mut frozen_subtree_roots, num_elements, element);
            num_elements += 1;
        }

        Self::new(frozen_subtree_roots, num_elements)
    }

    /// Appends one element. This will update `frozen_subtree_roots` to store new frozen root nodes
    /// and remove old nodes if they are now part of a larger frozen subtree.
    fn append_one(
        frozen_subtree_roots: &mut Vec<HashValue>,
        num_existing_elements: u64,
        element: HashValue,
    ) {
        // For example, this accumulator originally had N = 7 elements. Appending an element is
        // like adding one to this number N: 0b0111 + 1 = 0b1000. Every time we carry a bit to the
        // left we merge the rightmost two subtrees and compute their parent.
        // ```text
        //       A
        //     /   \
        //    /     \
        //   o       o       B
        //  / \     / \     / \
        // o   o   o   o   o   o   o
        // ```

        // First just append the element.
        frozen_subtree_roots.push(element);

        // Next, merge the last two subtrees into one. If `num_existing_elements` has N trailing
        // ones, the carry will happen N times.
        let num_trailing_ones = (!num_existing_elements).trailing_zeros();

        for _i in 0..num_trailing_ones {
            let right_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let left_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let parent_hash = MerkleTreeInternalNode::<H>::new(left_hash, right_hash).hash();
            frozen_subtree_roots.push(parent_hash);
        }
    }

    /// Computes the root hash of an accumulator given the frozen subtree roots.
    pub fn root_hash(&self) -> HashValue {
        if self.frozen_subtree_roots.is_empty() {
            return *ACCUMULATOR_PLACEHOLDER_HASH;
        }

        // First, start from the rightmost leaf position and move it to the rightmost frozen root.
        let max_leaf_index = self.num_elements - 1;
        let mut current_position = Position::from_leaf_index(max_leaf_index);
        // Move current position up until it reaches the corresponding frozen subtree root.
        while current_position.get_parent().is_freezable(max_leaf_index) {
            current_position = current_position.get_parent();
        }

        let mut roots = self.frozen_subtree_roots.iter().rev();
        let mut current_hash = *roots
            .next()
            .expect("We have checked frozen_subtree_roots is not empty.");

        // While current position is not root, find current sibling and compute parent hash.
        let root_position = Position::get_root_position(max_leaf_index);
        while current_position != root_position {
            current_hash = match current_position.get_direction_for_self() {
                NodeDirection::Left => {
                    // If a frozen node is the left child of its parent, its sibling must be
                    // a placeholder node.
                    MerkleTreeInternalNode::<H>::new(current_hash, *ACCUMULATOR_PLACEHOLDER_HASH)
                        .hash()
                }
                NodeDirection::Right => {
                    // Otherwise the left sibling must have been frozen.
                    MerkleTreeInternalNode::<H>::new(
                        *roots.next().expect("Ran out of subtree roots."),
                        current_hash,
                    )
                    .hash()
                }
            };
            current_position = current_position.get_parent();
        }

        current_hash
    }

    /// Returns the total number of elements in this accumulator.
    pub fn num_elements(&self) -> u64 {
        self.num_elements
    }
}

// We manually implement Debug because H (CryptoHasher) does not implement Debug.
impl<H> std::fmt::Debug for Accumulator<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Accumulator {{ frozen_subtree_roots: {:?}, num_elements: {:?} }}",
            self.frozen_subtree_roots, self.num_elements
        )
    }
}
