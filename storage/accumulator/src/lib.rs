// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

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

use crypto::hash::{CryptoHash, CryptoHasher, HashValue, ACCUMULATOR_PLACEHOLDER_HASH};
use failure::prelude::*;
use std::marker::PhantomData;
use types::proof::{
    position::{FrozenSubTreeIterator, FrozenSubtreeSiblingIterator, Position},
    AccumulatorConsistencyProof, AccumulatorProof, MerkleTreeInternalNode,
};

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
        num_existing_leaves: u64,
        new_leaves: &[HashValue],
    ) -> Result<(HashValue, Vec<Node>)> {
        MerkleAccumulatorView::<R, H>::new(reader, num_existing_leaves).append(new_leaves)
    }

    /// Get proof of inclusion of the leaf at `leaf_index` in this Merkle Accumulator of
    /// `num_leaves` leaves in total. Siblings are read via `reader` (or generated dynamically
    /// if they are non-frozen).
    ///
    /// See [`types::proof::AccumulatorProof`] for proof format.
    pub fn get_proof(reader: &R, num_leaves: u64, leaf_index: u64) -> Result<AccumulatorProof> {
        MerkleAccumulatorView::<R, H>::new(reader, num_leaves).get_proof(leaf_index)
    }

    /// Gets a proof that this accumulator is consistent with another accumulator that has
    /// `sub_acc_leaves` leaves. `sub_acc_leaves` must not be greater than `full_acc_leaves`.
    ///
    /// See [`types::proof::AccumulatorConsistencyProof`] for proof format.
    pub fn get_consistency_proof(
        reader: &R,
        full_acc_leaves: u64,
        sub_acc_leaves: u64,
    ) -> Result<AccumulatorConsistencyProof> {
        MerkleAccumulatorView::<R, H>::new(reader, full_acc_leaves)
            .get_consistency_proof(sub_acc_leaves)
    }
}

/// Actual implementation of Merkle Accumulator algorithms, which carries the `reader` and
/// `num_leaves` on an instance for convenience
struct MerkleAccumulatorView<'a, R, H> {
    reader: &'a R,
    num_leaves: u64,
    hasher: PhantomData<H>,
}

impl<'a, R, H> MerkleAccumulatorView<'a, R, H>
where
    R: HashReader,
    H: CryptoHasher,
{
    fn new(reader: &'a R, num_leaves: u64) -> Self {
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
                let root_hash = self.get_hash(Position::get_root_position(self.num_leaves - 1))?;
                return Ok((root_hash, Vec::new()));
            }
        }

        let num_new_leaves = new_leaves.len();
        let last_new_leaf_idx = self.num_leaves + num_new_leaves as u64 - 1;
        let root_level = Position::get_root_position(last_new_leaf_idx).get_level() as usize;
        let mut to_freeze = Vec::with_capacity(Self::max_to_freeze(num_new_leaves, root_level));

        // create one new node for each new leaf hash
        let mut current_level = self.gen_leaf_level(new_leaves);
        Self::record_to_freeze(
            &mut to_freeze,
            &current_level,
            false, /* has_non_frozen */
        );

        // loop starting from leaf level, upwards till root_level - 1,
        // making new nodes of parent level and recording frozen ones.
        let mut has_non_frozen = false;
        for _ in 0..root_level {
            let (parent_level, placeholder_used) = self.gen_parent_level(&current_level)?;

            // If a placeholder node is used to generate the right most node of a certain level,
            // such level and all its parent levels have a non-frozen right most node.
            has_non_frozen |= placeholder_used;
            Self::record_to_freeze(&mut to_freeze, &parent_level, has_non_frozen);

            current_level = parent_level;
        }

        assert_eq!(current_level.len(), 1, "must conclude in single root node");
        Ok((current_level.first().expect("unexpected None").1, to_freeze))
    }

    /// upper bound of num of frozen nodes:
    ///     new leaves and resulting frozen internal nodes forming a complete binary subtree
    ///         num_new_leaves * 2 - 1 < num_new_leaves * 2
    ///     and the full route from root of that subtree to the accumulator root turns frozen
    ///         height - (log2(num_new_leaves) + 1) < height - 1 = root_level
    fn max_to_freeze(num_new_leaves: usize, root_level: usize) -> usize {
        num_new_leaves * 2 + root_level
    }

    fn hash_internal_node(left: HashValue, right: HashValue) -> HashValue {
        MerkleTreeInternalNode::<H>::new(left, right).hash()
    }

    /// Given leaf level hashes, create leaf level nodes
    fn gen_leaf_level(&self, new_leaves: &[HashValue]) -> Vec<Node> {
        new_leaves
            .iter()
            .enumerate()
            .map(|(i, hash)| (Position::from_leaf_index(self.num_leaves + i as u64), *hash))
            .collect()
    }

    /// Given a level of new nodes (frozen or not), return new nodes on its parent level, and
    /// a boolean value indicating whether a placeholder node is used to construct the last node
    fn gen_parent_level(&self, current_level: &[Node]) -> Result<((Vec<Node>, bool))> {
        let mut parent_level: Vec<Node> = Vec::with_capacity(current_level.len() / 2 + 1);
        let mut iter = current_level.iter().peekable();

        // first node may be a right child, in that case pair it with its existing sibling
        let (first_pos, first_hash) = iter.peek().expect("Current level is empty");
        if !first_pos.is_left_child() {
            parent_level.push((
                first_pos.get_parent(),
                Self::hash_internal_node(self.reader.get(first_pos.get_sibling())?, *first_hash),
            ));
            iter.next();
        }

        // walk through in pairs of siblings, use placeholder as last right sibling if necessary
        let mut placeholder_used = false;
        while let Some((left_pos, left_hash)) = iter.next() {
            let right_hash = match iter.next() {
                Some((_, h)) => h,
                None => {
                    placeholder_used = true;
                    &ACCUMULATOR_PLACEHOLDER_HASH
                }
            };

            parent_level.push((
                left_pos.get_parent(),
                Self::hash_internal_node(*left_hash, *right_hash),
            ));
        }

        Ok((parent_level, placeholder_used))
    }

    /// append a level of new nodes into output vector, skip the last one if it's a non-frozen node
    fn record_to_freeze(to_freeze: &mut Vec<Node>, level: &[Node], has_non_frozen: bool) {
        to_freeze.extend(
            level
                .iter()
                .take(level.len() - has_non_frozen as usize)
                .cloned(),
        )
    }

    fn get_hash(&self, position: Position) -> Result<HashValue> {
        if position.is_placeholder(self.num_leaves - 1) {
            Ok(*ACCUMULATOR_PLACEHOLDER_HASH)
        } else if position.is_freezable(self.num_leaves - 1) {
            self.reader.get(position)
        } else {
            // non-frozen non-placeholder node
            Ok(Self::hash_internal_node(
                self.get_hash(position.get_left_child())?,
                self.get_hash(position.get_right_child())?,
            ))
        }
    }

    /// implementation for pub interface `MerkleAccumulator::get_proof`
    fn get_proof(&self, leaf_index: u64) -> Result<AccumulatorProof> {
        ensure!(
            leaf_index < self.num_leaves,
            "invalid leaf_index {}, num_leaves {}",
            leaf_index,
            self.num_leaves
        );

        let leaf_pos = Position::from_leaf_index(leaf_index);
        let root_pos = Position::get_root_position(self.num_leaves - 1);

        let siblings: Vec<HashValue> = leaf_pos
            .iter_ancestor_sibling()
            .take(root_pos.get_level() as usize)
            .map(|p| self.get_hash(p))
            .collect::<Result<Vec<HashValue>>>()?
            .into_iter()
            .rev()
            .collect();

        Ok(AccumulatorProof::new(siblings))
    }

    /// Implementation for public interface `MerkleAccumulator::get_consistency_proof`.
    fn get_consistency_proof(&self, sub_acc_leaves: u64) -> Result<AccumulatorConsistencyProof> {
        ensure!(
            sub_acc_leaves <= self.num_leaves,
            "The other accumulator is bigger than this one. self.num_leaves: {}. \
             sub_acc_leaves: {}.",
            self.num_leaves,
            sub_acc_leaves,
        );

        // If the other accumulator is empty. Nothing is needed for the proof since any accumulator
        // is consistent with an empty one.
        if sub_acc_leaves == 0 {
            return Ok(AccumulatorConsistencyProof::new(vec![], vec![]));
        }

        let frozen_subtree_roots = FrozenSubTreeIterator::new(sub_acc_leaves)
            .map(|p| self.get_hash(p))
            .collect::<Result<Vec<_>>>()?;

        let root_level = Position::get_root_position(self.num_leaves - 1).get_level();
        let siblings = FrozenSubtreeSiblingIterator::new(sub_acc_leaves)
            .take_while(|p| p.get_level() < root_level)
            .map(|p| self.get_hash(p))
            .collect::<Result<Vec<_>>>()?;

        Ok(AccumulatorConsistencyProof::new(
            frozen_subtree_roots,
            siblings,
        ))
    }
}

#[cfg(test)]
mod tests;
