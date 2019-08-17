// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements an in-memory Merkle Accumulator that is similar to what we use in
//! storage. This accumulator will only store a small portion of the tree -- for any subtree that
//! is full, we store only the root. Also we only store the frozen nodes, therefore this structure
//! will always store up to `Log(n)` number of nodes, where `n` is the total number of leaves in
//! the tree.
//!
//! This accumulator is immutable once constructed. If we append new leaves to the tree we will
//! obtain a new accumulator instance and the old one remains unchanged.

#[cfg(test)]
mod accumulator_test;

use crate::proof::MerkleTreeInternalNode;
use crypto::{
    hash::{CryptoHash, CryptoHasher, ACCUMULATOR_PLACEHOLDER_HASH},
    HashValue,
};
use failure::prelude::*;
use std::marker::PhantomData;

/// The Accumulator implementation.
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

    /// The total number of leaves in this accumulator.
    num_leaves: u64,

    /// The root hash of this accumulator.
    root_hash: HashValue,

    phantom: PhantomData<H>,
}

impl<H> Accumulator<H>
where
    H: CryptoHasher,
{
    /// Constructs a new accumulator with roots of existing frozen subtrees. Returns error if the
    /// number of frozen subtree roots does not match the number of leaves.
    pub fn new(frozen_subtree_roots: Vec<HashValue>, num_leaves: u64) -> Result<Self> {
        ensure!(
            frozen_subtree_roots.len() == num_leaves.count_ones() as usize,
            "The number of frozen subtrees does not match the number of leaves. \
             frozen_subtree_roots.len(): {}. num_leaves: {}.",
            frozen_subtree_roots.len(),
            num_leaves,
        );

        let root_hash = Self::compute_root_hash(&frozen_subtree_roots, num_leaves);

        Ok(Self {
            frozen_subtree_roots,
            num_leaves,
            root_hash,
            phantom: PhantomData,
        })
    }

    /// Appends a list of new leaves to an existing accumulator. Since the accumulator is
    /// immutable, the existing one remains unchanged and a new one representing the result is
    /// returned.
    pub fn append(&self, leaves: Vec<HashValue>) -> Self {
        let mut frozen_subtree_roots = self.frozen_subtree_roots.clone();
        let mut num_leaves = self.num_leaves;
        for leaf in leaves {
            Self::append_one(&mut frozen_subtree_roots, num_leaves, leaf);
            num_leaves += 1;
        }

        Self::new(frozen_subtree_roots, num_leaves).expect(
            "Appending leaves to a valid accumulator should create another valid accumulator.",
        )
    }

    /// Appends one leaf. This will update `frozen_subtree_roots` to store new frozen root nodes
    /// and remove old nodes if they are now part of a larger frozen subtree.
    fn append_one(
        frozen_subtree_roots: &mut Vec<HashValue>,
        num_existing_leaves: u64,
        leaf: HashValue,
    ) {
        // For example, this accumulator originally had N = 7 leaves. Appending a leaf is like
        // adding one to this number N: 0b0111 + 1 = 0b1000. Every time we carry a bit to the left
        // we merge the rightmost two subtrees and compute their parent.
        // ```text
        //       A
        //     /   \
        //    /     \
        //   o       o       B
        //  / \     / \     / \
        // o   o   o   o   o   o   o
        // ```

        // First just append the leaf.
        frozen_subtree_roots.push(leaf);

        // Next, merge the last two subtrees into one. If `num_existing_leaves` has N trailing
        // ones, the carry will happen N times.
        let num_trailing_ones = (!num_existing_leaves).trailing_zeros();

        for _i in 0..num_trailing_ones {
            let right_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let left_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let parent_hash = MerkleTreeInternalNode::<H>::new(left_hash, right_hash).hash();
            frozen_subtree_roots.push(parent_hash);
        }
    }

    /// Computes what the root hash would become if the current accumulator is extended with a list
    /// of new leaves. This is similar to [`append`](Accumulator::append) except that the new
    /// leaves themselves are not known and they are represented by `siblings`, so we would not be
    /// able to construct a new list of frozen subtrees, but would just compute the root hash of
    /// the new accumulator. As an example, given the following accumulator that currently has 10
    /// leaves, the frozen subtree roots and the siblings are annotated below. Note that in this
    /// case `siblings[0]` represents two new leaves `A` and `B`, `siblings[1]` represents four new
    /// leaves, and the last `siblings[2]` may represent 1~16 new leaves.
    ///
    /// ```text
    ///                                                 new_root
    ///                                                /        \
    ///                                               /          \
    ///                                       old_root            siblings[2]
    ///                                      /        \
    ///                                    /            \
    ///                                  /                \
    ///                                /                    \
    ///                              /                        \
    ///                            /                            \
    ///                          /                                \
    ///                        /                                    \
    /// frozen_subtree_roots[0]                                      o
    ///                    /   \                                    / \
    ///                   /     \                                  /   \
    ///                  o       o                                o     siblings[1]
    ///                 / \     / \                              / \
    ///                o   o   o   o      frozen_subtree_roots[1]   siblings[0]
    ///               / \ / \ / \ / \                         / \           / \
    ///               o o o o o o o o                         o o           A B
    /// ```
    pub fn append_siblings(&self, siblings: &[HashValue]) -> Result<HashValue> {
        Self::append_siblings_impl(&self.frozen_subtree_roots, self.num_leaves, siblings)
    }

    fn append_siblings_impl(
        frozen_subtree_roots: &[HashValue],
        num_existing_leaves: u64,
        siblings: &[HashValue],
    ) -> Result<HashValue> {
        if frozen_subtree_roots.is_empty() {
            ensure!(
                siblings.len() == 1,
                "{} siblings are provided when only one is required.",
                siblings.len(),
            );
            return Ok(siblings[0]);
        }
        if siblings.is_empty() {
            ensure!(
                frozen_subtree_roots.len() == 1,
                "No siblings are provided when some are required to compute root hash."
            );
            return Ok(frozen_subtree_roots[0]);
        }

        Self::validate_siblings(num_existing_leaves, siblings)?;

        let mut frozen_subtree_iter = frozen_subtree_roots.iter().rev().peekable();
        let mut sibling_iter = siblings.iter().peekable();
        let mut current_hash = *sibling_iter
            .next()
            .expect("Code would have returned if siblings is empty.");

        // Using the above example, num_existing_leaves = 10 = 0b1010. We check each bit from right
        // to left starting from the rightmost one bit. If a bit is 0, it means there is no
        // existing frozen subtree on this level, so we combine current hash with a sibling.
        // Otherwise we combine it with the corresponding frozen subtree.
        let mut bitmap = num_existing_leaves >> num_existing_leaves.trailing_zeros();
        while frozen_subtree_iter.peek().is_some() || sibling_iter.peek().is_some() {
            current_hash = if bitmap & 1 == 0 {
                MerkleTreeInternalNode::<H>::new(
                    current_hash,
                    *sibling_iter.next().expect("This sibling should exist."),
                )
            } else {
                MerkleTreeInternalNode::<H>::new(
                    *frozen_subtree_iter
                        .next()
                        .expect("This frozen subtree should exist."),
                    current_hash,
                )
            }
            .hash();
            bitmap >>= 1;
        }
        assert_eq!(bitmap, 0);

        Ok(current_hash)
    }

    /// Does some basic validation on the input siblings.
    fn validate_siblings(num_existing_leaves: u64, siblings: &[HashValue]) -> Result<()> {
        // `siblings` should start with 0 or more non-placeholder trees, followed by 0 or more
        // placeholder ones.
        for i in 1..siblings.len() {
            if siblings[i] != *ACCUMULATOR_PLACEHOLDER_HASH {
                ensure!(
                    siblings[i - 1] != *ACCUMULATOR_PLACEHOLDER_HASH,
                    "Placeholder sibling should not be followed by non-placeholder sibling.",
                );
            }
        }

        let max_num_siblings =
            num_existing_leaves.count_zeros() - num_existing_leaves.trailing_zeros();
        ensure!(
            siblings.len() <= max_num_siblings as usize,
            "Max number of siblings: {}. {} are provided.",
            max_num_siblings,
            siblings.len(),
        );
        let min_num_siblings = max_num_siblings - num_existing_leaves.leading_zeros();
        ensure!(
            siblings.len() >= min_num_siblings as usize,
            "Min number of siblings: {}. {} are provided.",
            min_num_siblings,
            siblings.len(),
        );

        Ok(())
    }

    /// Returns the root hash of the accumulator.
    pub fn root_hash(&self) -> HashValue {
        self.root_hash
    }

    /// Computes the root hash of an accumulator given the frozen subtree roots and the number of
    /// leaves in this accumulator.
    fn compute_root_hash(frozen_subtree_roots: &[HashValue], num_leaves: u64) -> HashValue {
        if frozen_subtree_roots.is_empty() {
            return *ACCUMULATOR_PLACEHOLDER_HASH;
        }

        // Computing the root hash is equivalent to calling `append_siblings` with a list of
        // placeholder siblings.

        // For any accumulator, if we add two children to every existing leaf, the positions of the
        // frozen subtrees do no change, so the trailing zeros do not matter.
        let num_leaves = num_leaves >> num_leaves.trailing_zeros();
        // Because every time we combine two hashes to compute its parent, the tree can be made one
        // level smaller. So the total number of hashes needed to compute the root hash is
        // `num_levels`.
        let num_levels = num_leaves.next_power_of_two().trailing_zeros() + 1;
        let num_placeholders = num_levels as usize - frozen_subtree_roots.len();

        Self::append_siblings_impl(
            frozen_subtree_roots,
            num_leaves,
            &vec![*ACCUMULATOR_PLACEHOLDER_HASH; num_placeholders],
        )
        .expect("Computing root hash should succeed.")
    }

    /// Returns the total number of leaves in this accumulator.
    pub fn num_leaves(&self) -> u64 {
        self.num_leaves
    }
}

// We manually implement Debug because H (CryptoHasher) does not implement Debug.
impl<H> std::fmt::Debug for Accumulator<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Accumulator {{ frozen_subtree_roots: {:?}, num_leaves: {:?} }}",
            self.frozen_subtree_roots, self.num_leaves
        )
    }
}

impl<H> Default for Accumulator<H>
where
    H: CryptoHasher,
{
    fn default() -> Self {
        Accumulator::new(vec![], 0).expect("Constructing empty accumulator should work.")
    }
}
