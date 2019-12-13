// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an abstraction for positioning a node in a binary tree,
//! A `Position` uniquely identifies the location of a node
//!
//! In this implementation, `Position` is represented by the in-order-traversal sequence number
//! of the node.  The process of locating a node and jumping between nodes is done through
//! in-order position calculation, which can be done with bit manipulation.
//!
//! For example
//! ```text
//!      3
//!     /  \
//!    /    \
//!   1      5 <-[Node index, a.k.a, Position]
//!  / \    / \
//! 0   2  4   6
//!
//! 0   1  2   3 <[Leaf index]
//! ```
//! Note1: The in-order-traversal counts from 0
//! Note2: The level of tree counts from leaf level, start from 0
//! Note3: The leaf index starting from left-most leaf, starts from 0

use crate::proof::definition::{LeafCount, MAX_ACCUMULATOR_LEAVES, MAX_ACCUMULATOR_PROOF_DEPTH};
use mirai_annotations::*;
use std::fmt;

#[cfg(test)]
mod position_test;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Position(u64);
// invariant Position.0 < u64::max_value() - 1

#[derive(Debug, Eq, PartialEq)]
pub enum NodeDirection {
    Left,
    Right,
}

impl Position {
    /// What level is this node in the tree, 0 if the node is a leaf,
    /// 1 if the level is one above a leaf, etc.
    pub fn level(self) -> u32 {
        (!self.0).trailing_zeros()
    }

    fn is_leaf(self) -> bool {
        self.0 & 1 == 0
    }

    /// What position is the node within the level? i.e. how many nodes
    /// are to the left of this node at the same level
    #[cfg(test)]
    fn pos_counting_from_left(self) -> u64 {
        self.0 >> (self.level() + 1)
    }

    /// pos count start from 0 on each level
    pub fn from_level_and_pos(level: u32, pos: u64) -> Self {
        precondition!(level < 63);
        assume!(1u64 << level > 0); // bitwise and integer operations don't mix.
        let level_one_bits = (1u64 << level) - 1;
        let shifted_pos = pos << (level + 1);
        Position(shifted_pos | level_one_bits)
    }

    pub fn from_inorder_index(index: u64) -> Self {
        Position(index)
    }

    pub fn to_inorder_index(self) -> u64 {
        self.0
    }

    pub fn from_postorder_index(index: u64) -> Self {
        Position(postorder_to_inorder(index))
    }

    pub fn to_postorder_index(self) -> u64 {
        inorder_to_postorder(self.to_inorder_index())
    }

    /// What is the parent of this node?
    pub fn parent(self) -> Self {
        assume!(self.0 < u64::max_value() - 1); // invariant
        Self(
            (self.0 | isolate_rightmost_zero_bit(self.0))
                & !(isolate_rightmost_zero_bit(self.0) << 1),
        )
    }

    /// What is the left node of this node? Will overflow if the node is a leaf
    pub fn left_child(self) -> Self {
        checked_precondition!(!self.is_leaf());
        Self::child(self, NodeDirection::Left)
    }

    /// What is the right node of this node? Will overflow if the node is a leaf
    pub fn right_child(self) -> Self {
        checked_precondition!(!self.is_leaf());
        Self::child(self, NodeDirection::Right)
    }

    fn child(self, dir: NodeDirection) -> Self {
        checked_precondition!(!self.is_leaf());
        assume!(self.0 < u64::max_value() - 1); // invariant

        let direction_bit = match dir {
            NodeDirection::Left => 0,
            NodeDirection::Right => isolate_rightmost_zero_bit(self.0),
        };
        Self((self.0 | direction_bit) & !(isolate_rightmost_zero_bit(self.0) >> 1))
    }

    /// Whether this position is a left child of its parent.  The observation is that,
    /// after stripping out all right-most 1 bits, a left child will have a bit pattern
    /// of xxx00(11..), while a right child will be represented by xxx10(11..)
    pub fn is_left_child(self) -> bool {
        assume!(self.0 < u64::max_value() - 1); // invariant
        self.0 & (isolate_rightmost_zero_bit(self.0) << 1) == 0
    }

    pub fn is_right_child(self) -> bool {
        !self.is_left_child()
    }

    // Opposite of get_left_node_count_from_position.
    pub fn from_leaf_index(leaf_index: u64) -> Self {
        Self::from_level_and_pos(0, leaf_index)
    }

    /// This method takes in a node position and return its sibling position
    ///
    /// The observation is that, after stripping out the right-most common bits,
    /// two sibling nodes flip the the next right-most bits with each other.
    /// To find out the right-most common bits, first remove all the right-most ones
    /// because they are corresponding to level's indicator. Then remove next zero right after.
    pub fn sibling(self) -> Self {
        assume!(self.0 < u64::max_value() - 1); // invariant
        Self(self.0 ^ (isolate_rightmost_zero_bit(self.0) << 1))
    }

    // Given a leaf index, calculate the position of a minimum root which contains this leaf
    /// This method calculates the index of the smallest root which contains this leaf.
    /// Observe that, the root position is composed by a "height" number of ones
    ///
    /// For example
    /// ```text
    ///     0010010(node)
    ///     0011111(smearing)
    ///     -------
    ///     0001111(root)
    /// ```
    pub fn root_from_leaf_index(leaf_index: u64) -> Self {
        let leaf = Self::from_leaf_index(leaf_index);
        Self(smear_ones_for_u64(leaf.0) >> 1)
    }

    pub fn root_from_leaf_count(leaf_count: LeafCount) -> Self {
        assert!(leaf_count > 0);
        Self::root_from_leaf_index((leaf_count - 1) as u64)
    }

    pub fn root_level_from_leaf_count(leaf_count: LeafCount) -> u32 {
        assert!(leaf_count > 0);
        let index = (leaf_count - 1) as u64;
        MAX_ACCUMULATOR_PROOF_DEPTH as u32 + 1 - index.leading_zeros()
    }

    /// Given a node, find its right most child in its subtree.
    /// Right most child is a Position, could be itself, at level 0
    pub fn right_most_child(self) -> Self {
        let level = self.level();
        Self(self.0 + (1_u64 << level) - 1)
    }

    /// Given a node, find its left most child in its subtree
    /// Left most child is a node, could be itself, at level 0
    pub fn left_most_child(self) -> Self {
        // Turn off its right most x bits. while x=level of node
        let level = self.level();
        Self(turn_off_right_most_n_bits(self.0, level))
    }
}

// Some helper functions to perform general bit manipulation

/// Smearing all the bits starting from MSB with ones
fn smear_ones_for_u64(v: u64) -> u64 {
    let mut n = v;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n
}

/// Turn off n right most bits
///
/// For example
/// ```text
///     00010010101
///     -----------
///     00010010100 n=1
///     00010010000 n=3
/// ```
fn turn_off_right_most_n_bits(v: u64, n: u32) -> u64 {
    precondition!(n < 64);
    (v >> n) << n
}

/// Finds the rightmost 0-bit, turns off all bits, and sets this bit to 1 in
/// the result. For example:
///
/// ```text
///     01110111  (x)
///     --------
///     10001000  (~x)
/// &   01111000  (x+1)
///     --------
///     00001000
/// ```
/// http://www.catonmat.net/blog/low-level-bit-hacks-you-absolutely-must-know/
fn isolate_rightmost_zero_bit(v: u64) -> u64 {
    !v & (v + 1)
}

// The following part of the position implementation is logically separate and
// depends on our notion of freezable.  It should probably move to another module.
impl Position {
    // Given index of right most leaf, calculate if a position is the root
    // of a perfect subtree that does not contain any placeholder nodes.
    //
    // First find its right most child
    // the right most child of any node will be at leaf level, which will be a either placeholder
    // node or leaf node. if right most child is a leaf node, then it is freezable.
    // if right most child is larger than max_leaf_node, it is a placeholder node, and not
    // freezable.
    pub fn is_freezable(self, leaf_index: u64) -> bool {
        let leaf = Self::from_leaf_index(leaf_index);
        let right_most_child = self.right_most_child();
        right_most_child.0 <= leaf.0
    }

    // Given index of right most leaf, calculate if a position should contain
    // a placeholder node at this moment
    // A node is a placeholder if both two conditions below are true:
    // 1, the node's in order traversal seq > max_leaf_node's, and
    // 2, the node does not have left child or right child.
    pub fn is_placeholder(self, leaf_index: u64) -> bool {
        let leaf = Self::from_leaf_index(leaf_index);
        if self.0 <= leaf.0 {
            return false;
        }
        if self.left_most_child().0 <= leaf.0 {
            return false;
        }
        true
    }

    /// Creates an `AncestorIterator` using this position.
    pub fn iter_ancestor(self) -> AncestorIterator {
        AncestorIterator { position: self }
    }

    /// Creates an `AncestorSiblingIterator` using this position.
    pub fn iter_ancestor_sibling(self) -> AncestorSiblingIterator {
        AncestorSiblingIterator { position: self }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos({})", self.to_inorder_index())
    }
}

/// `AncestorSiblingIterator` generates current sibling position and moves itself to its parent
/// position for each iteration.
#[derive(Debug)]
pub struct AncestorSiblingIterator {
    position: Position,
}

impl Iterator for AncestorSiblingIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        let current_sibling_position = self.position.sibling();
        self.position = self.position.parent();
        Some(current_sibling_position)
    }
}

/// `AncestorIterator` generates current position and moves itself to its parent position for each
/// iteration.
#[derive(Debug)]
pub struct AncestorIterator {
    position: Position,
}

impl Iterator for AncestorIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        let current_position = self.position;
        self.position = self.position.parent();
        Some(current_position)
    }
}

/// Traverse leaves from left to right in groups that forms full subtrees, yielding root positions
/// of such subtrees.
/// Note that each 1-bit in num_leaves corresponds to a full subtree.
/// For example, in the below tree of 5=0b101 leaves, the two 1-bits corresponds to Fzn2 and L4
/// accordingly.
///
/// ```text
///            Non-fzn
///           /       \
///          /         \
///         /           \
///       Fzn2         Non-fzn
///      /   \           /   \
///     /     \         /     \
///    Fzn1    Fzn3  Non-fzn  [Placeholder]
///   /  \    /  \    /    \
///  L0  L1  L2  L3 L4   [Placeholder]
/// ```
pub struct FrozenSubTreeIterator {
    bitmap: u64,
    seen_leaves: u64,
    // invariant seen_leaves < u64::max_value() - bitmap
}

impl FrozenSubTreeIterator {
    pub fn new(num_leaves: LeafCount) -> Self {
        Self {
            bitmap: num_leaves,
            seen_leaves: 0,
        }
    }
}

impl Iterator for FrozenSubTreeIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        assume!(self.seen_leaves < u64::max_value() - self.bitmap); // invariant

        if self.bitmap == 0 {
            return None;
        }

        // Find the remaining biggest full subtree.
        // The MSB of the bitmap represents it. For example for a tree of 0b1010=10 leaves, the
        // biggest and leftmost full subtree has 0b1000=8 leaves, which can be got by smearing all
        // bits after MSB with 1-bits (got 0b1111), right shift once (got 0b0111) and add 1 (got
        // 0b1000=8). At the same time, we also observe that the in-order numbering of a full
        // subtree root is (num_leaves - 1) greater than that of the leftmost leaf, and also
        // (num_leaves - 1) less than that of the rightmost leaf.
        let root_offset = smear_ones_for_u64(self.bitmap) >> 1;
        assume!(root_offset < self.bitmap); // relate bit logic to integer logic
        let num_leaves = root_offset + 1;
        let leftmost_leaf = Position::from_leaf_index(self.seen_leaves);
        let root = Position::from_inorder_index(leftmost_leaf.to_inorder_index() + root_offset);

        // Mark it consumed.
        self.bitmap &= !num_leaves;
        self.seen_leaves += num_leaves;

        Some(root)
    }
}

/// Given an accumulator of size `current_num_leaves`, `FrozenSubtreeSiblingIterator` yields the
/// positions of required subtrees if we want to append these subtrees to the existing accumulator
/// to generate a bigger one of size `new_num_leaves`.
///
/// See [`crate::proof::accumulator::Accumulator::append_subtrees`] for more details.
pub struct FrozenSubtreeSiblingIterator {
    current_num_leaves: LeafCount,
    remaining_new_leaves: LeafCount,
}

impl FrozenSubtreeSiblingIterator {
    /// Constructs a new `FrozenSubtreeSiblingIterator` given the size of current accumulator and
    /// the size of the bigger accumulator.
    pub fn new(current_num_leaves: LeafCount, new_num_leaves: LeafCount) -> Self {
        assert!(
            new_num_leaves <= MAX_ACCUMULATOR_LEAVES,
            "An accumulator can have at most 2^{} leaves. Provided num_leaves: {}.",
            MAX_ACCUMULATOR_PROOF_DEPTH,
            new_num_leaves,
        );
        assert!(
            current_num_leaves <= new_num_leaves,
            "Number of leaves needs to be increasing: current_num_leaves: {}, new_num_leaves: {}",
            current_num_leaves,
            new_num_leaves
        );

        Self {
            current_num_leaves,
            remaining_new_leaves: new_num_leaves - current_num_leaves,
        }
    }

    /// Helper function to return the next set of leaves that form a complete subtree.  For
    /// example, if there are 5 leaves (..0101), 2 ^ (63 - 61 leading zeros) = 4 leaves should be
    /// taken next.
    fn next_new_leaf_batch(&self) -> LeafCount {
        let zeros = self.remaining_new_leaves.leading_zeros();
        1 << (MAX_ACCUMULATOR_PROOF_DEPTH - zeros as usize)
    }
}

impl Iterator for FrozenSubtreeSiblingIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_new_leaves == 0 {
            return None;
        }

        // Now we compute the size of the next subtree. If there is a rightmost frozen subtree, we
        // may combine it with a subtree of the same size, or append a smaller one on the right. In
        // case self.current_num_leaves is zero and there is no rightmost frozen subtree, the
        // largest possible one is appended.
        let next_subtree_leaves = if self.current_num_leaves > 0 {
            let rightmost_frozen_subtree_leaves = 1 << self.current_num_leaves.trailing_zeros();
            if self.remaining_new_leaves >= rightmost_frozen_subtree_leaves {
                rightmost_frozen_subtree_leaves
            } else {
                self.next_new_leaf_batch()
            }
        } else {
            self.next_new_leaf_batch()
        };

        // Now that the size of the next subtree is known, we compute the leftmost and rightmost
        // leaves in this subtree. The root of the subtree is then the middle of these two leaves.
        let first_leaf_index = self.current_num_leaves;
        let last_leaf_index = first_leaf_index + next_subtree_leaves - 1;
        self.current_num_leaves += next_subtree_leaves;
        self.remaining_new_leaves -= next_subtree_leaves;

        Some(Position::from_inorder_index(
            (first_leaf_index + last_leaf_index) as u64,
        ))
    }
}

fn children_of_node(node: u64) -> u64 {
    (isolate_rightmost_zero_bit(node) << 1) - 2
}

/// In a post-order tree traversal, how many nodes are traversed before `node`
/// not including nodes that are children of `node`.
fn nodes_to_left_of(node: u64) -> u64 {
    // If node = 0b0100111, ones_up_to_level = 0b111
    let ones_up_to_level = isolate_rightmost_zero_bit(node) - 1;
    // Unset all the 1s due to the level
    let unset_level_zeros = node ^ ones_up_to_level;

    // What remains is a 1 bit set every time a node navigated right
    // For example, consider node=5=0b101. unset_level_zeros=0b100.
    // the 1 bit in unset_level_zeros at position 2 represents the
    // fact that 5 is the right child at the level 1. At this level
    // there are 2^2 - 1 children on the left hand side.
    //
    // So what we do is subtract the count of one bits from unset_level_zeros
    // to account for the fact that if the node is the right child at level
    // n that there are 2^n - 1 children.
    unset_level_zeros - u64::from(unset_level_zeros.count_ones())
}

/// Given `node`, an index in an in-order traversal of a perfect binary tree,
/// what order would the node be visited in in post-order traversal?
/// For example, consider this tree of in-order nodes.
///
/// ```text
///      3
///     /  \
///    /    \
///   1      5
///  / \    / \
/// 0   2  4   6
/// ```
///
/// The post-order ordering of the nodes is:
/// ```text
///      6
///     /  \
///    /    \
///   2      5
///  / \    / \
/// 0   1  3   4
/// ```
///
/// post_order_index(1) == 2
/// post_order_index(4) == 3
pub fn inorder_to_postorder(node: u64) -> u64 {
    let children = children_of_node(node);
    let left_nodes = nodes_to_left_of(node);

    children + left_nodes
}

pub fn postorder_to_inorder(mut node: u64) -> u64 {
    // The number of nodes in a full binary tree with height `n` is `2^n - 1`.
    let mut full_binary_size = !0u64;
    let mut bitmap = 0u64;
    for i in (0..64).rev() {
        if node >= full_binary_size {
            node -= full_binary_size;
            bitmap |= 1 << i;
        }
        full_binary_size >>= 1;
    }
    let level = node as u32;
    let pos = bitmap >> level;
    Position::from_level_and_pos(level, pos).to_inorder_index()
}
