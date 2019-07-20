// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides functions to manipulate a perfect binary tree. Nodes
//! in the tree are represented by the order they would be visited in an
//! in-order traversal. For example --
//! ```text
//!      3
//!     /  \
//!    /    \
//!   1      5
//!  / \    / \
//! 0   2  4   6
//! ```
//!
//! This module can answer questions like "what is the level of 5"
//! (`level(5)=1`), "what is the right child of 3" `right_child(3)=5`
#[derive(Debug, Eq, PartialEq)]
pub enum NodeDirection {
    Left,
    Right,
}

/// What level is this node in the tree, 0 if the node is a leaf,
/// 1 if the level is one above a leaf, etc.
pub fn level(node: u64) -> u32 {
    (!node).trailing_zeros()
}

fn is_leaf(node: u64) -> bool {
    node & 1 == 0
}

/// What position is the node within the level? i.e. how many nodes
/// are to the left of this node at the same level
pub fn pos_counting_from_left(node: u64) -> u64 {
    node >> (level(node) + 1)
}

/// pos count start from 0 on each level
pub fn node_from_level_and_pos(level: u32, pos: u64) -> u64 {
    let level_one_bits = (1u64 << level) - 1;
    let shifted_pos = pos << (level + 1);
    shifted_pos | level_one_bits
}

/// What is the parent of this node?
pub fn parent(node: u64) -> u64 {
    (node | isolate_rightmost_zero_bit(node)) & !(isolate_rightmost_zero_bit(node) << 1)
}

/// What is the left node of this node? Will overflow if the node is a leaf
pub fn left_child(node: u64) -> u64 {
    child(node, NodeDirection::Left)
}

/// What is the right node of this node? Will overflow if the node is a leaf
pub fn right_child(node: u64) -> u64 {
    child(node, NodeDirection::Right)
}

pub fn child(node: u64, dir: NodeDirection) -> u64 {
    assert!(!is_leaf(node));

    let direction_bit = match dir {
        NodeDirection::Left => 0,
        NodeDirection::Right => isolate_rightmost_zero_bit(node),
    };
    (node | direction_bit) & !(isolate_rightmost_zero_bit(node) >> 1)
}

/// This method takes in a node position and return NodeDirection based on if it's left or right
/// child Similar to sibling. The observation is that,
/// after strip out the right-most common bits,
/// if next right-most bits is 0, it is left child. Otherwise, right child
pub fn direction_from_parent(node: u64) -> NodeDirection {
    match node & (isolate_rightmost_zero_bit(node) << 1) {
        0 => NodeDirection::Left,
        _ => NodeDirection::Right,
    }
}

/// This method takes in a node position and return its sibling position
///
/// The observation is that, after stripping out the right-most common bits,
/// two sibling nodes flip the the next right-most bits with each other.
/// To find out the right-most common bits, first remove all the right-most ones
/// because they are corresponding to level's indicator. Then remove next zero right after.
pub fn sibling(node: u64) -> u64 {
    node ^ (isolate_rightmost_zero_bit(node) << 1)
}

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
pub fn get_root(node: u64) -> u64 {
    smear_ones_for_u64(node) >> 1
}

/// Smearing all the bits starting from MSB with ones
pub fn smear_ones_for_u64(v: u64) -> u64 {
    let mut n = v;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n
}

/// Returns the number of children a node `level` nodes high in a perfect
/// binary tree has.
///
/// Recursively,
///
/// children_from_level(0) = 0
/// children_from_level(n) = 2 * (1 + children(n-1))
///
/// But expanding the series this can be computed non-recursively
/// sum 2^n, n=1 to x = 2^(x+1) - 2
pub fn children_from_level(level: u32) -> u64 {
    (1u64 << (level + 1)) - 2
}

pub fn children_of_node(node: u64) -> u64 {
    (isolate_rightmost_zero_bit(node) << 1) - 2
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

/// In a post-order tree traversal, how many nodes are traversed before `node`
/// not including nodes that are children of `node`.
pub fn nodes_to_left_of(node: u64) -> u64 {
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

/// This method checks if a node is freezable.
/// A freezable is a node with a perfect subtree that does not include any placeholder node.
///
/// First find its right most child
/// the right most child of any node will be at leaf level, which will be a either placeholder node
/// or leaf node. if right most child is a leaf node, then it is freezable.
/// if right most child is larger than max_leaf_node, it is a placeholder node, and not freezable.
pub fn is_freezable(node: u64, max_leaf_node: u64) -> bool {
    let right_most_child = right_most_child(node);
    right_most_child <= max_leaf_node
}

/// This method checks if a node is a placeholder node.
/// A node is a placeholder if both two conditions below are true:
/// 1, the node's in order traversal seq > max_leaf_node's, and
/// 2, the node does not have left child or right child.
pub fn is_placeholder(node: u64, max_leaf_node: u64) -> bool {
    if node <= max_leaf_node {
        return false;
    }
    if left_most_child(node) <= max_leaf_node {
        return false;
    }
    true
}

/// Given a node, find its right most child in its subtree.
/// Right most child is a node, could be itself, at level 0
pub fn right_most_child(node: u64) -> u64 {
    let level = level(node);
    node + (1_u64 << level) - 1
}

/// Given a node, find its left most child in its subtree
/// Left most child is a node, could be itself, at level 0
pub fn left_most_child(node: u64) -> u64 {
    // Turn off its right most x bits. while x=level of node
    let level = level(node);
    turn_off_right_most_n_bits(node, level)
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
    (v >> n) << n
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
    node_from_level_and_pos(level, pos)
}
