// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an abstraction for positioning a node in a binary tree,
//! A `Position` uniquely identifies the location of a node
//!
//! In this implementation, `Position` is represented by the in-order-traversal sequence number
//! of the node.
//! The process of locating a node and jumping between nodes is done through position calculation,
//! which comes from treebits.
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

use super::treebits;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Position(u64);

impl Position {
    pub fn from_inorder_index(index: u64) -> Self {
        Position(index)
    }

    pub fn to_inorder_index(self) -> u64 {
        self.0
    }

    pub fn get_parent(self) -> Self {
        Self::from_inorder_index(treebits::parent(self.0))
    }

    // Note: if self is root, the sibling will overflow
    pub fn get_sibling(self) -> Self {
        Self::from_inorder_index(treebits::sibling(self.0))
    }

    // Requirement: self can not be leaf.
    pub fn get_left_child(self) -> Position {
        Self::from_inorder_index(treebits::left_child(self.0))
    }

    // Requirement: self can not be leaf.
    pub fn get_right_child(self) -> Position {
        Self::from_inorder_index(treebits::right_child(self.0))
    }

    // Note: if self is root, the direction will overflow (and will always be left)
    pub fn get_direction_for_self(self) -> treebits::NodeDirection {
        treebits::direction_from_parent(self.0)
    }

    // The level start from 0 counting from the leaf level
    pub fn get_level(self) -> u32 {
        treebits::level(self.0)
    }

    // Given the position, return the leaf index counting from the left
    pub fn to_leaf_index(self) -> u64 {
        treebits::pos_counting_from_left(self.0)
    }

    // Opposite of get_left_node_count_from_position.
    pub fn from_leaf_index(leaf_index: u64) -> Position {
        Self::from_inorder_index(treebits::node_from_level_and_pos(0, leaf_index))
    }

    /// Given a position, returns the position next to it on the right on the same level. For
    /// example, given input 5 this function should return 9.
    ///
    /// ```text
    ///       3
    ///    /     \
    ///   1       5       9
    ///  / \     / \     / \
    /// 0   2   4   6   8   10
    /// ```
    pub fn get_next_sibling(self) -> Position {
        let level = self.get_level();
        let pos = treebits::pos_counting_from_left(self.0);
        Position(treebits::node_from_level_and_pos(level, pos + 1))
    }

    // Given a leaf index, calculate the position of a minimum root which contains this leaf
    pub fn get_root_position(leaf_index: u64) -> Position {
        let leaf = Self::from_leaf_index(leaf_index);
        Self::from_inorder_index(treebits::get_root(leaf.0))
    }

    // Given index of right most leaf, calculate if a position is the root
    // of a perfect subtree that does not contains placeholder nodes.
    pub fn is_freezable(self, leaf_index: u64) -> bool {
        let leaf = Self::from_leaf_index(leaf_index);
        treebits::is_freezable(self.0, leaf.0)
    }

    // Given index of right most leaf, calculate if a position should be a placeholder node at this
    // moment
    pub fn is_placeholder(self, leaf_index: u64) -> bool {
        let leaf = Self::from_leaf_index(leaf_index);
        treebits::is_placeholder(self.0, leaf.0)
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

/// `AncestorSiblingIterator` generates current sibling position and moves itself to its parent
/// position for each iteration.
#[derive(Debug)]
pub struct AncestorSiblingIterator {
    position: Position,
}

impl Iterator for AncestorSiblingIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        let current_sibling_position = self.position.get_sibling();
        self.position = self.position.get_parent();
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
        self.position = self.position.get_parent();
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
}

impl FrozenSubTreeIterator {
    pub fn new(num_leaves: u64) -> Self {
        Self {
            bitmap: num_leaves,
            seen_leaves: 0,
        }
    }
}

impl Iterator for FrozenSubTreeIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        if self.bitmap == 0 {
            return None;
        }

        // Find the remaining biggest full subtree.
        // The MSB of the bitmap represents it. For example for a tree of 0x1010=10 leaves, the
        // biggest and leftmost full subtree has 0x1000=8 leaves, which can be got by smearing all
        // bits after MSB with 1-bits (got 0x1111), right shift once (got 0x0111) and add 1 (got
        // 0x1000=8). At the same time, we also observe that the in-order numbering of a full
        // subtree root is (num_leaves - 1) greater than that of the leftmost leaf, and also
        // (num_leaves - 1) less than that of the rightmost leaf.
        let root_offset = treebits::smear_ones_for_u64(self.bitmap) >> 1;
        let num_leaves = root_offset + 1;
        let leftmost_leaf = Position::from_leaf_index(self.seen_leaves);
        let root = Position::from_inorder_index(leftmost_leaf.to_inorder_index() + root_offset);

        // Mark it consumed.
        self.bitmap &= !num_leaves;
        self.seen_leaves += num_leaves;

        Some(root)
    }
}
