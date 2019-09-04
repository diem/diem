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

#[cfg(test)]
mod position_test;
mod treebits;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Position(u64);

impl Position {
    pub fn from_inorder_index(index: u64) -> Self {
        Position(index)
    }

    pub fn to_inorder_index(self) -> u64 {
        self.0
    }

    pub fn from_postorder_index(index: u64) -> Self {
        Self::from_inorder_index(treebits::postorder_to_inorder(index))
    }

    pub fn to_postorder_index(self) -> u64 {
        treebits::inorder_to_postorder(self.to_inorder_index())
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

    /// Whether this position is a left child of its parent.
    pub fn is_left_child(self) -> bool {
        match treebits::direction_from_parent(self.0) {
            treebits::NodeDirection::Left => true,
            treebits::NodeDirection::Right => false,
        }
    }

    // The level start from 0 counting from the leaf level
    pub fn get_level(self) -> u32 {
        treebits::level(self.0)
    }

    // Compute the position given the level and the pos count on this level.
    pub fn from_level_and_pos(level: u32, pos: u64) -> Self {
        Self::from_inorder_index(treebits::node_from_level_and_pos(level, pos))
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
        // The MSB of the bitmap represents it. For example for a tree of 0b1010=10 leaves, the
        // biggest and leftmost full subtree has 0b1000=8 leaves, which can be got by smearing all
        // bits after MSB with 1-bits (got 0b1111), right shift once (got 0b0111) and add 1 (got
        // 0b1000=8). At the same time, we also observe that the in-order numbering of a full
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

/// Given an accumulator of size `current_num_leaves`, `FrozenSubtreeSiblingIterator` yields the
/// positions of required subtrees if we want to append these subtrees to the existing accumulator
/// to generate a bigger one of size `new_num_leaves`.
///
/// See [`crate::proof::accumulator::Accumulator`] for more details.
pub struct FrozenSubtreeSiblingIterator {
    current_num_leaves: u64,
    remaining_new_leaves: u64,
}

impl FrozenSubtreeSiblingIterator {
    /// Constructs a new `FrozenSubtreeSiblingIterator` given the size of current accumulator and
    /// the size of the bigger accumulator.
    pub fn new(current_num_leaves: u64, new_num_leaves: u64) -> Self {
        assert!(
            new_num_leaves <= 1 << 63,
            "An accumulator can have at most 2^63 leaves. Provided num_leaves: {}.",
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
        let next_subtree_size = if self.current_num_leaves > 0 {
            let rightmost_frozen_subtree_size = 1 << self.current_num_leaves.trailing_zeros();
            if self.remaining_new_leaves >= rightmost_frozen_subtree_size {
                rightmost_frozen_subtree_size
            } else {
                1 << (63 - self.remaining_new_leaves.leading_zeros())
            }
        } else {
            1 << (63 - self.remaining_new_leaves.leading_zeros())
        };

        // Now that the size of the next subtree is known, we compute the leftmost and rightmost
        // leaves in this subtree. The root of the subtree is then the middle of these two leaves.
        let first_leaf_index = self.current_num_leaves;
        let last_leaf_index = first_leaf_index + next_subtree_size - 1;
        self.current_num_leaves += next_subtree_size;
        self.remaining_new_leaves -= next_subtree_size;

        Some(Position::from_inorder_index(
            first_leaf_index + last_leaf_index,
        ))
    }
}
