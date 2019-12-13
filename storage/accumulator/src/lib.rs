// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This module provides algorithms for accessing and updating a Merkle Accumulator structure
//! persisted in a key-value store. Note that this doesn't write to the storage directly, rather,
//! it reads from it via the `HashReader` trait and yields writes via an in memory `HashMap`.
//!
//! # Merkle Accumulator
//! Given an ever growing (append only) series of "leaf" hashes, we construct an evolving Merkle
//! Tree for which proofs of inclusion/exclusion of a leaf hash at a leaf index in a snapshot
//! of the tree (represented by root hash) can be given.
//!
//! # Leaf Nodes
//! Leaf nodes carry hash values to be stored and proved. They are only appended to the tree but
//! never deleted or updated.
//!
//! # Internal Nodes
//! A non-leaf node carries the hash value derived from both its left and right children.
//!
//! # Placeholder Nodes
//! To make sure each Leaf node has a Merkle Proof towards the root, placeholder nodes are added so
//! that along the route from a leaf to the root, each node has a sibling. Placeholder nodes have
//! the hash value `ACCUMULATOR_PLACEHOLDER_HASH`
//!
//! A placeholder node can appear as either a Leaf node or a non-Leaf node, but there is at most one
//! placeholder leaf at any time.
//!
//! # Frozen Nodes & Non-frozen Nodes
//! As leaves are added to the tree, placeholder nodes get replaced by non-placeholder nodes, and
//! when a node has all its descendants being non-placeholder, it becomes "Frozen" -- its hash value
//! won't change again in the event of new leaves being added. All leaves appended (not counting the
//! one possible placeholder leaf) are by definition Frozen.
//!
//! Other nodes, which have one or more placeholder descendants are Non-Frozen. As new elements are
//! appended to the accumulator the hash value of these nodes will change.
//!
//! # Leaf Count
//! Given a count of the number of leaves in a Merkle Accumulator it is possible to determine the
//! shape of the accumulator -- which nodes are filled and which nodes are placeholder nodes.
//!
//! Example:
//! Logical view of a Merkle Accumulator with 5 leaves:
//! ```text
//!            Non-fzn
//!           /       \
//!          /         \
//!         /           \
//!       Fzn2         Non-fzn
//!      /   \           /   \
//!     /     \         /     \
//!    Fzn1    Fzn3  Non-fzn  [Placeholder]
//!   /  \    /  \    /    \
//!  L0  L1  L2  L3 L4   [Placeholder]
//! ```
//!
//! # Position and Physical Representation
//! As a Merkle Accumulator tree expands to the right and upwards, we number newly frozen nodes
//! monotonically. One way to do it is simply to use in-order index of nodes, and this is what
//! we do for the in-memory representation. We call the stated numbers identifying nodes below
//! simply "Position", and unless otherwise stated, this is the in-order position.
//!
//! For writing to disk however, we write all the children of a node before the parent.
//! Thus for disk write order, it is more convenient to use the post-order position as an index.
//! And with that we can map a Merkle Accumulator into a key-value storage: key is the post-order
//! position of a node, and the value is hash value it carries.
//!
//! We store only Frozen nodes, and generate non-Frozen nodes on the fly when accessing the tree.
//! This way, the physical representation of the tree is append-only, i.e. once written to physical
//! storage, nodes won't be either modified or deleted.
//!
//! Here is what we persist for the logical tree in the above example:
//!
//! ```text
//!          Fzn2(6)
//!         /      \
//!        /        \
//!    Fzn1(2)       Fzn3(5)
//!   /     \       /     \
//!  L0(0)  L1(1)  L2(3)  L3(4)  L4(7)
//! ```
//!
//! When the next leaf node is persisted, the physical representation will be:
//!
//! ```text
//!          Fzn2(6)
//!         /      \
//!        /        \
//!    Fzn1(2)       Fzn3(5)       Fzn4(9)
//!   /     \       /     \       /      \
//!  L0(0)  L1(1)  L2(3)  L3(4)  L4(7)   L5(8)
//! ```
//!
//! The numbering corresponds to the post-order traversal of the tree.
//!
//! To think in key-value pairs:
//! ```text
//! |<-key->|<--value-->|
//! |   0   | hash_L0   |
//! |   1   | hash_L1   |
//! |   2   | hash_Fzn1 |
//! |  ...  |   ...     |
//! ```

use anyhow::{ensure, format_err, Result};
use libra_crypto::hash::{CryptoHash, CryptoHasher, HashValue, ACCUMULATOR_PLACEHOLDER_HASH};
use libra_types::proof::{
    definition::{LeafCount, MAX_ACCUMULATOR_PROOF_DEPTH},
    position::{FrozenSubTreeIterator, FrozenSubtreeSiblingIterator, Position},
    AccumulatorConsistencyProof, AccumulatorProof, AccumulatorRangeProof, MerkleTreeInternalNode,
};
use mirai_annotations::*;
use std::marker::PhantomData;

/// Defines the interface between `MerkleAccumulator` and underlying storage.
pub trait HashReader {
    /// Return `HashValue` carried by the node at `Position`.
    fn get(&self, position: Position) -> Result<HashValue>;
}

/// A `Node` in a `MerkleAccumulator` tree is a `HashValue` at a `Position`
type Node = (Position, HashValue);

/// In this live Merkle Accumulator algorithms.
pub struct MerkleAccumulator<R, H> {
    reader: PhantomData<R>,
    hasher: PhantomData<H>,
}

impl<R, H> MerkleAccumulator<R, H>
where
    R: HashReader,
    H: CryptoHasher,
{
    /// Given an existing Merkle Accumulator (represented by `num_existing_leaves` and a `reader`
    /// that is able to fetch all existing frozen nodes), and a list of leaves to be appended,
    /// returns the result root hash and new nodes to be frozen.
    pub fn append(
        reader: &R,
        num_existing_leaves: LeafCount,
        new_leaves: &[HashValue],
    ) -> Result<(HashValue, Vec<Node>)> {
        MerkleAccumulatorView::<R, H>::new(reader, num_existing_leaves).append(new_leaves)
    }

    /// Get proof of inclusion of the leaf at `leaf_index` in this Merkle Accumulator of
    /// `num_leaves` leaves in total. Siblings are read via `reader` (or generated dynamically
    /// if they are non-frozen).
    ///
    /// See [`libra_types::proof::AccumulatorProof`] for proof format.
    pub fn get_proof(
        reader: &R,
        num_leaves: LeafCount,
        leaf_index: u64,
    ) -> Result<AccumulatorProof<H>> {
        MerkleAccumulatorView::<R, H>::new(reader, num_leaves).get_proof(leaf_index)
    }

    /// Gets a proof that shows the full accumulator is consistent with a smaller accumulator.
    ///
    /// See [`libra_types::proof::AccumulatorConsistencyProof`] for proof format.
    pub fn get_consistency_proof(
        reader: &R,
        full_acc_leaves: LeafCount,
        sub_acc_leaves: LeafCount,
    ) -> Result<AccumulatorConsistencyProof> {
        MerkleAccumulatorView::<R, H>::new(reader, full_acc_leaves)
            .get_consistency_proof(sub_acc_leaves)
    }

    /// Gets a proof that shows a range of leaves are part of the accumulator.
    ///
    /// See [`libra_types::proof::AccumulatorRangeProof`] for proof format.
    pub fn get_range_proof(
        reader: &R,
        full_acc_leaves: LeafCount,
        first_leaf_index: Option<u64>,
        num_leaves: LeafCount,
    ) -> Result<AccumulatorRangeProof<H>> {
        MerkleAccumulatorView::<R, H>::new(reader, full_acc_leaves)
            .get_range_proof(first_leaf_index, num_leaves)
    }

    /// From left to right, gets frozen subtree root hashes of the accumulator. For example, if the
    /// accumulator has 5 leaves, `x` and `e` are returned.
    /// ```text
    ///                 root
    ///                /    \
    ///              /        \
    ///            /            \
    ///           x              o
    ///         /   \           / \
    ///        /     \         /   \
    ///       o       o       o     placeholder
    ///      / \     / \     / \
    ///     a   b   c   d   e   placeholder
    /// ```
    pub fn get_frozen_subtree_hashes(reader: &R, num_leaves: LeafCount) -> Result<Vec<HashValue>> {
        MerkleAccumulatorView::<R, H>::new(reader, num_leaves).get_frozen_subtree_hashes()
    }
}

/// Actual implementation of Merkle Accumulator algorithms, which carries the `reader` and
/// `num_leaves` on an instance for convenience
struct MerkleAccumulatorView<'a, R, H> {
    reader: &'a R,
    num_leaves: LeafCount,
    hasher: PhantomData<H>,
}

impl<'a, R, H> MerkleAccumulatorView<'a, R, H>
where
    R: HashReader,
    H: CryptoHasher,
{
    fn new(reader: &'a R, num_leaves: LeafCount) -> Self {
        Self {
            reader,
            num_leaves,
            hasher: PhantomData,
        }
    }

    /// implementation for pub interface `MerkleAccumulator::append`
    fn append(&self, new_leaves: &[HashValue]) -> Result<(HashValue, Vec<Node>)> {
        // Deal with the case where new_leaves is empty
        if new_leaves.is_empty() {
            if self.num_leaves == 0 {
                return Ok((*ACCUMULATOR_PLACEHOLDER_HASH, Vec::new()));
            } else {
                let root_hash = self.get_hash(Position::root_from_leaf_count(self.num_leaves))?;
                return Ok((root_hash, Vec::new()));
            }
        }

        let num_new_leaves = new_leaves.len();
        let last_new_leaf_count = self.num_leaves + num_new_leaves as LeafCount;
        let root_level = Position::root_level_from_leaf_count(last_new_leaf_count);
        let mut to_freeze = Vec::with_capacity(Self::max_to_freeze(num_new_leaves, root_level));

        // Iterate over the new leaves, adding them to to_freeze and then adding any frozen parents
        // when right children are encountered.  This has the effect of creating frozen nodes in
        // perfect post-order, which can be used as a strictly increasing append only index for
        // the underlying storage.
        //
        // We will track newly created left siblings while iterating so we can pair them with their
        // right sibling, if and when it becomes frozen.  If the frozen left sibling is not created
        // in this iteration, it must already exist in storage.
        let mut left_siblings: Vec<(_, _)> = Vec::new();
        for (leaf_offset, leaf) in new_leaves.iter().enumerate() {
            let leaf_pos = Position::from_leaf_index(self.num_leaves + leaf_offset as LeafCount);
            let mut hash = *leaf;
            to_freeze.push((leaf_pos, hash));
            let mut pos = leaf_pos;
            while pos.is_right_child() {
                let sibling = pos.sibling();
                hash = match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        Self::hash_internal_node(left_hash, hash)
                    }
                    None => Self::hash_internal_node(self.reader.get(sibling)?, hash),
                };
                pos = pos.parent();
                to_freeze.push((pos, hash));
            }
            // The node remaining must be a left child, possibly the root of a complete binary tree.
            left_siblings.push((pos, hash));
        }

        // Now reconstruct the final root hash by walking up to root level and adding
        // placeholder hash nodes as needed on the right, and left siblings that have either
        // been newly created or read from storage.
        let (mut pos, mut hash) = left_siblings.pop().expect("Must have at least one node");
        for _ in pos.level()..root_level as u32 {
            hash = if pos.is_left_child() {
                Self::hash_internal_node(hash, *ACCUMULATOR_PLACEHOLDER_HASH)
            } else {
                let sibling = pos.sibling();
                match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        Self::hash_internal_node(left_hash, hash)
                    }
                    None => Self::hash_internal_node(self.reader.get(sibling)?, hash),
                }
            };
            pos = pos.parent();
        }
        assert!(left_siblings.is_empty());

        Ok((hash, to_freeze))
    }

    /// upper bound of num of frozen nodes:
    ///     new leaves and resulting frozen internal nodes forming a complete binary subtree
    ///         num_new_leaves * 2 - 1 < num_new_leaves * 2
    ///     and the full route from root of that subtree to the accumulator root turns frozen
    ///         height - (log2(num_new_leaves) + 1) < height - 1 = root_level
    fn max_to_freeze(num_new_leaves: usize, root_level: u32) -> usize {
        precondition!(root_level as usize <= MAX_ACCUMULATOR_PROOF_DEPTH);
        precondition!(num_new_leaves < (usize::max_value() / 2));
        precondition!(num_new_leaves * 2 <= usize::max_value() - root_level as usize);
        num_new_leaves * 2 + root_level as usize
    }

    fn hash_internal_node(left: HashValue, right: HashValue) -> HashValue {
        MerkleTreeInternalNode::<H>::new(left, right).hash()
    }

    fn rightmost_leaf_index(&self) -> u64 {
        (self.num_leaves - 1) as u64
    }

    fn get_hash(&self, position: Position) -> Result<HashValue> {
        let idx = self.rightmost_leaf_index();
        if position.is_placeholder(idx) {
            Ok(*ACCUMULATOR_PLACEHOLDER_HASH)
        } else if position.is_freezable(idx) {
            self.reader.get(position)
        } else {
            // non-frozen non-placeholder node
            Ok(Self::hash_internal_node(
                self.get_hash(position.left_child())?,
                self.get_hash(position.right_child())?,
            ))
        }
    }

    /// implementation for pub interface `MerkleAccumulator::get_proof`
    fn get_proof(&self, leaf_index: u64) -> Result<AccumulatorProof<H>> {
        ensure!(
            leaf_index < self.num_leaves as u64,
            "invalid leaf_index {}, num_leaves {}",
            leaf_index,
            self.num_leaves
        );

        let siblings = self.get_siblings(leaf_index, |_p| true)?;
        Ok(AccumulatorProof::new(siblings))
    }

    /// Implementation for public interface `MerkleAccumulator::get_consistency_proof`.
    fn get_consistency_proof(
        &self,
        sub_acc_leaves: LeafCount,
    ) -> Result<AccumulatorConsistencyProof> {
        ensure!(
            sub_acc_leaves <= self.num_leaves,
            "The other accumulator is bigger than this one. self.num_leaves: {}. \
             sub_acc_leaves: {}.",
            self.num_leaves,
            sub_acc_leaves,
        );

        let subtrees = FrozenSubtreeSiblingIterator::new(sub_acc_leaves, self.num_leaves)
            .map(|p| self.reader.get(p))
            .collect::<Result<Vec<_>>>()?;

        Ok(AccumulatorConsistencyProof::new(subtrees))
    }

    /// Implementation for public interface `MerkleAccumulator::get_range_proof`.
    fn get_range_proof(
        &self,
        first_leaf_index: Option<u64>,
        num_leaves: LeafCount,
    ) -> Result<AccumulatorRangeProof<H>> {
        if first_leaf_index.is_none() {
            ensure!(
                num_leaves == 0,
                "num_leaves is not zero while first_leaf_index is None.",
            );
            return Ok(AccumulatorRangeProof::new_empty());
        }

        let first_leaf_index = first_leaf_index.expect("first_leaf_index should not be None.");
        ensure!(
            num_leaves > 0,
            "num_leaves is zero while first_leaf_index is not None.",
        );
        let last_leaf_index = first_leaf_index
            .checked_add(num_leaves - 1)
            .ok_or_else(|| format_err!("Requesting too many leaves."))?;
        ensure!(
            last_leaf_index < self.num_leaves as u64,
            "Invalid last_leaf_index: {}, num_leaves: {}",
            last_leaf_index,
            self.num_leaves,
        );

        let left_siblings = self.get_siblings(first_leaf_index, |p| p.is_left_child())?;
        let right_siblings = self.get_siblings(last_leaf_index, |p| p.is_right_child())?;
        Ok(AccumulatorRangeProof::new(left_siblings, right_siblings))
    }

    /// Helper function to get siblings on the path from the given leaf to the root. An additional
    /// filter function can be applied to filter out certain siblings.
    fn get_siblings(
        &self,
        leaf_index: u64,
        filter: impl Fn(Position) -> bool,
    ) -> Result<Vec<HashValue>> {
        let root_pos = Position::root_from_leaf_count(self.num_leaves);
        let siblings = Position::from_leaf_index(leaf_index)
            .iter_ancestor_sibling()
            .take(root_pos.level() as usize)
            .filter_map(|p| {
                if filter(p) {
                    Some(self.get_hash(p))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(siblings)
    }

    /// Implementation for public interface `MerkleAccumulator::get_frozen_subtree_hashes`.
    fn get_frozen_subtree_hashes(&self) -> Result<Vec<HashValue>> {
        FrozenSubTreeIterator::new(self.num_leaves)
            .map(|p| self.reader.get(p))
            .collect::<Result<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests;
