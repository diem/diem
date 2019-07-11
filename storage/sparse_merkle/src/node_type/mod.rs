// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Node types of [`SparseMerkleTree`](crate::SparseMerkleTree)
//!
//! This module defines three types of patricia Merkle tree nodes: [`BranchNode`],
//! [`ExtensionNode`] and [`LeafNode`] as building blocks of a 256-bit
//! [`SparseMerkleTree`](crate::SparseMerkleTree). [`BranchNode`] represents a 4-level binary tree
//! to optimize for IOPS: it compresses a tree with 31 nodes into one node with 16 chidren at the
//! lowest level. [`ExtensionNode`] compresses a partial path without any fork into a single node by
//! storing the partial path inside. [`LeafNode`] stores the full key and the value hash which is
//! used as the key to query binary account blob data from the storage.

#[cfg(test)]
mod node_type_test;

use crate::nibble_path::{skip_common_prefix, NibbleIterator, NibblePath};
use bincode::{deserialize, serialize};
use crypto::{
    hash::{
        CryptoHash, SparseMerkleInternalHasher, SparseMerkleLeafHasher,
        SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
use failure::{Fail, Result};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use types::proof::{SparseMerkleInternalNode, SparseMerkleLeafNode};

pub(crate) type Children = HashMap<u8, (HashValue, bool)>;

/// Represents a 4-level subtree with 16 children at the bottom level. Theoretically, this reduces
/// IOPS to query a tree by 4x since we compress 4 levels in a standard Merkle tree into 1 node.
/// Though we choose the same branch node structure as that of a patricia Merkle tree, the root hash
/// computation logic is similar to a 4-level sparse Merkle tree except for some customizations. See
/// the `CryptoHash` trait implementation below for details.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BranchNode {
    // key: child index from 0 to 15, inclusive.
    // value: Child node hash and a boolean whose true value indicates the child is a leaf node.
    children: Children,
}

/// Node in a patricia Merkle tree. It compresses a path without any fork with a single
/// node instead of multiple single-child branch nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionNode {
    // The nibble path this extension node encapsulates.
    nibble_path: NibblePath,
    // Represents the next node down the path.
    child: HashValue,
}

/// Represents an account. It has two fields: `key` is the hash of the acccont address and
/// `value_hash` is the hash of account state blob.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeafNode {
    // the full key of this node
    key: HashValue,
    // the hash of the data blob identified by the key
    value_hash: HashValue,
}

/// The explicit tag is used as a prefix in the encoded format of nodes to distinguish different
/// node discrinminants.
trait Tag {
    const TAG: u8;
}

// We leave 0 reserved.
impl Tag for BranchNode {
    const TAG: u8 = 1;
}

impl Tag for ExtensionNode {
    const TAG: u8 = 2;
}

impl Tag for LeafNode {
    const TAG: u8 = 3;
}

/// Computes the hash of branch node according to [`SparseMerkleTree`](crate::SparseMerkleTree)
/// data structure in the logical view. `start` and `nibble_height` determine a subtree whose
/// root hash we want to get. For a branch node with 16 children at the bottom level, we compute
/// the root hash of it as if a full binary Merkle tree with 16 leaves as below:
///
/// ```text
/// 
///   4 ->              +------ root hash ------+
///                     |                       |
///   3 ->        +---- # ----+           +---- # ----+
///               |           |           |           |
///   2 ->        #           #           #           #
///             /   \       /   \       /   \       /   \
///   1 ->     #     #     #     #     #     #     #     #
///           / \   / \   / \   / \   / \   / \   / \   / \
///   0 ->   0   1 2   3 4   5 6   7 8   9 A   B C   D E   F
///   ^
/// height
/// ```
///
/// As illustrated above, at nibble height 0, `0..F` in hex denote 16 chidren hashes.  Each `#`
/// means the hash of its two direct children, which will be used to generate the hash of its
/// parent with the hash of its sibling. Finally, we can get the hash of this branch node.
///
/// However, if a branch node doesn't have all 16 chidren exist at height 0 but just a few of
/// them, we have a modified hashing rule on top of what is stated above:
/// 1. From top to bottom, a node will be replaced by a leaf child if the subtree rooted at this
/// node has only one child at height 0 and it is a leaf child.
/// 2. From top to bottom, a node will be replaced by the placeholder node if the subtree rooted at
/// this node doesn't have any child at height 0. For example, if a branch node has 3 leaf nodes
/// at index 0, 3, 8, respectively, and 1 branch/extension node at index C, then the computation
/// graph will be like:
///
/// ```text
/// 
///   4 ->              +------ root hash ------+
///                     |                       |
///   3 ->        +---- # ----+           +---- # ----+
///               |           |           |           |
///   2 ->        #           @           8           #
///             /   \                               /   \
///   1 ->     0     3                             #     @
///                                               / \
///   0 ->                                       C   @
///   ^
/// height
/// Note: @ denotes placeholder hash.
/// ```
impl CryptoHash for BranchNode {
    // Unused hasher.
    type Hasher = SparseMerkleInternalHasher;

    fn hash(&self) -> HashValue {
        self.merkle_hash(
            0,  /* start index */
            16, /* the number of leaves in the subtree of which we want the hash of root */
            self.generate_bitmaps(),
        )
    }
}

/// Computes the hash of an [`ExtensionNode`]. Similar to [`BranchNode`], we generate
/// the hash by logically expanding it into a sparse Merkle tree.  For an extension node with 2
/// nibbles, compute the final hash as follows:
///
/// ```text
/// 
///                   #(final hash)
///                  / \
///                 #   placeholder
///                / \
///               #   placeholder
///              / \
///    placeholder  #
///                / \
///               #   placeholder
///              / \
///    placeholder  \
///                / \
///               #   placeholder
///              / \
///             #   placeholder
///            / \
///       child   placeholder
/// ```
///
/// The final hash is generated by iteratively hashing the concatenation of two children of each
/// node following a bottom-up order.  It is worth nothing that by definition [`ExtensionNode`] is
/// just a path, so each intermediate node must only have one child. When being expanded to a
/// sparse Merkle tree logically, empty nodes should be replaced by the default digest.
impl CryptoHash for ExtensionNode {
    // Unused hasher.
    type Hasher = SparseMerkleInternalHasher;

    fn hash(&self) -> HashValue {
        self.nibble_path.bits().rev().fold(self.child, |hash, bit| {
            if bit {
                SparseMerkleInternalNode::new(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash).hash()
            } else {
                SparseMerkleInternalNode::new(hash, *SPARSE_MERKLE_PLACEHOLDER_HASH).hash()
            }
        })
    }
}

/// Computes the hash of a [`LeafNode`].
impl CryptoHash for LeafNode {
    // Unused hasher.
    type Hasher = SparseMerkleLeafHasher;

    fn hash(&self) -> HashValue {
        SparseMerkleLeafNode::new(self.key, self.value_hash).hash()
    }
}

/// The concrete node type of [`SparseMerkleTree`](crate::SparseMerkleTree).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Node {
    /// A wrapper of [`BranchNode`].
    Branch(BranchNode),
    /// A wrapper of [`ExtensionNode`].
    Extension(ExtensionNode),
    /// A wrapper of [`LeafNode`].
    Leaf(LeafNode),
}

impl From<BranchNode> for Node {
    fn from(node: BranchNode) -> Self {
        Node::Branch(node)
    }
}

impl From<ExtensionNode> for Node {
    fn from(node: ExtensionNode) -> Self {
        Node::Extension(node)
    }
}

impl From<LeafNode> for Node {
    fn from(node: LeafNode) -> Self {
        Node::Leaf(node)
    }
}

impl BranchNode {
    /// Creates a new branch node.
    pub fn new(children: HashMap<u8, (HashValue, bool)>) -> Self {
        Self { children }
    }

    /// Sets the `n`-th child to given hash and stores a `bool` indicating whether the child passed
    /// in is a leaf node.
    pub fn set_child(&mut self, n: u8, child: (HashValue, bool)) {
        assert!(n < 16);
        self.children.insert(n, child);
    }

    /// Gets the hash of the `n`-th child.
    pub fn child(&self, n: u8) -> Option<HashValue> {
        assert!(n < 16);
        self.children.get(&n).map(|p| p.0)
    }

    /// Returns an `Option<bool>` indicating whether the `n`-th child is a leaf node. If the child
    /// doesn't exist, returns `None`.
    pub fn is_leaf(&self, n: u8) -> Option<bool> {
        assert!(n < 16);
        self.children.get(&n).map(|p| p.1)
    }

    /// Return the total number of children
    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    /// Generates `child_bitmap` and `leaf_bitmap` as a pair of `u16`: child at index `i` exists if
    /// `child_bitmap[i]` is set; child at index `i` is leaf node if `leaf_bitmap[i]` is set.
    pub fn generate_bitmaps(&self) -> (u16, u16) {
        let mut child_bitmap = 0_u16;
        let mut leaf_bitmap = 0_u16;
        for (k, v) in self.children.iter() {
            child_bitmap |= 1u16 << k;
            leaf_bitmap |= (v.1 as u16) << k;
        }
        // `leaf_bitmap` must be a subset of `child_bitmap`.
        assert!(child_bitmap | leaf_bitmap == child_bitmap);
        (child_bitmap, leaf_bitmap)
    }

    /// Given a range [start, start + width), returns the sub-bitmap of that range.
    fn range_bitmaps(start: u8, width: u8, bitmaps: (u16, u16)) -> (u16, u16) {
        assert!(start < 16 && start % width == 0);
        // A range with `start == 8` and `width == 4` will generate a mask 0b0000111100000000.
        let mask = if width == 16 {
            0xffff
        } else {
            assert!(width <= 16 && (width & (width - 1)) == 0);
            (1 << width) - 1
        } << start;
        (bitmaps.0 & mask, bitmaps.1 & mask)
    }

    fn merkle_hash(
        &self,
        start: u8,
        width: u8,
        (child_bitmap, leaf_bitmap): (u16, u16),
    ) -> HashValue {
        // Given a bit [start, 1 << nibble_height], return the value of that range.
        let (range_child_bitmap, range_leaf_bitmap) =
            BranchNode::range_bitmaps(start, width, (child_bitmap, leaf_bitmap));
        if range_child_bitmap == 0 {
            // No child under this subtree
            *SPARSE_MERKLE_PLACEHOLDER_HASH
        } else if range_child_bitmap & (range_child_bitmap - 1) == 0
            && (range_leaf_bitmap != 0 || width == 1)
        {
            // Only 1 leaf child under this subtree or reach the lowest level
            let only_child_index = range_child_bitmap.trailing_zeros() as u8;
            self.children
                .get(&only_child_index)
                .unwrap_or_else(|| {
                    panic!(
                        "Corrupted branch node: child_bitmap indicates \
                         the existence of a non-exist child at index {}",
                        only_child_index
                    )
                })
                .0
        } else {
            let left_child = self.merkle_hash(start, width / 2, (child_bitmap, leaf_bitmap));
            let right_child =
                self.merkle_hash(start + width / 2, width / 2, (child_bitmap, leaf_bitmap));
            SparseMerkleInternalNode::new(left_child, right_child).hash()
        }
    }

    /// Gets the child and its corresponding siblings that are necessary to generate the proof for
    /// the `n`-th child. If it is an existence proof, the returned child must be the `n`-th
    /// child; otherwise, the returned child may be another child. See inline explanation for
    /// details. When calling this function with n = 11 (node `b` in the following graph), the
    /// range at each level is illustrated as a pair of square brackets:
    ///
    /// ```text                           
    ///     4      [f   e   d   c   b   a   9   8   7   6   5   4   3   2   1   0] -> root level
    ///            ---------------------------------------------------------------
    ///     3      [f   e   d   c   b   a   9   8] [7   6   5   4   3   2   1   0] width = 8
    ///                                  chs <--┘                        shs <--┘
    ///     2      [f   e   d   c] [b   a   9   8] [7   6   5   4] [3   2   1   0] width = 4
    ///                  shs <--┘               └--> chs
    ///     1      [f   e] [d   c] [b   a] [9   8] [7   6] [5   4] [3   2] [1   0] width = 2
    ///                          chs <--┘       └--> shs
    ///     0      [f] [e] [d] [c] [b] [a] [9] [8] [7] [6] [5] [4] [3] [2] [1] [0] width = 1
    ///     ^                chs <--┘   └--> shs
    ///     |   MSB|<---------------------- uint 16 ---------------------------->|LSB
    ///  height    chs: `child_half_start`         shs: `sibling_half_start`
    /// ```
    pub fn get_child_for_proof_and_siblings(&self, n: u8) -> (Option<HashValue>, Vec<HashValue>) {
        let mut siblings = vec![];
        assert!(n < 16);
        let (child_bitmap, leaf_bitmap) = self.generate_bitmaps();

        // Nibble height from 3 to 0.
        for h in (0..4).rev() {
            // Get the number of children of the branch node that each subtree at this height
            // covers.
            let width = 1 << h;
            // Get the index of the first child belonging to the same subtree whose root, let's say
            // `r` is at height `h` that the n-th child belongs to.
            // Note:  `child_half_start` will be always equal to `n` at height 0.
            let child_half_start = (0xff << h) & n;
            // Get the index of the first child belonging to the subtree whose root is the sibling
            // of `r` at height `h`.
            let sibling_half_start = child_half_start ^ (1 << h);
            // Compute the root hash of the subtree rooted at the sibling of `r`.
            siblings.push(self.merkle_hash(sibling_half_start, width, (child_bitmap, leaf_bitmap)));

            let (range_child_bitmap, range_leaf_bitmap) =
                BranchNode::range_bitmaps(child_half_start, width, (child_bitmap, leaf_bitmap));

            if range_child_bitmap == 0 {
                // No child in this range.
                return (None, siblings);
            } else if range_child_bitmap.count_ones() == 1
                && (range_leaf_bitmap.count_ones() == 1 || width == 1)
            {
                // Return the only 1 leaf child under this subtree or reach the lowest level
                // Even this leaf child is not the n-th child, it should be returned instead of
                // `None` because it's existence indirectly proves the n-th child doesn't exist.
                // Please read proof format for details.
                let only_child_index = range_child_bitmap.trailing_zeros() as u8;
                return (
                    Some(
                        self.children
                            .get(&only_child_index)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Corrupted branch node: child_bitmap indicates \
                                     the existence of a non-exist child at index {}",
                                    only_child_index
                                )
                            })
                            .0,
                    ),
                    siblings,
                );
            }
        }
        unreachable!()
    }
}

impl ExtensionNode {
    /// Creates a new extension node.
    pub fn new(nibble_path: NibblePath, child: HashValue) -> Self {
        Self { nibble_path, child }
    }

    /// Gets the only child.
    pub fn child(&self) -> HashValue {
        self.child
    }

    /// Sets the child.
    pub fn set_child(&mut self, child_hash: HashValue) {
        self.child = child_hash;
    }

    /// Gets the `encoded_path`.
    pub fn nibble_path(&self) -> &NibblePath {
        &self.nibble_path
    }

    /// Gets the siblings from this extension node according to the requested nibble iterator.
    /// Also return a boolean indicating whether we can stop traversing and return early.
    pub fn get_siblings(&self, nibble_iter: &mut NibbleIterator) -> (Vec<HashValue>, bool) {
        let mut extension_nibble_iter = self.nibble_path().nibbles();
        let mut siblings = vec![
            *SPARSE_MERKLE_PLACEHOLDER_HASH;
            skip_common_prefix(&mut extension_nibble_iter, nibble_iter) * 4 /* 1 nibble == 4 bits */
        ];
        // There are two possible cases after matching prefix:
        // 1. Not all the nibbles of the extension node match the nibble path of the queried key.
        // This means the queried key meets a default node when being matched with the extension
        // node nibble path, so we can terminate the search early and return a non-existence proof
        // with the proper number of siblings.
        if !extension_nibble_iter.is_finished() {
            let mut extension_bit_iter = extension_nibble_iter.bits();
            let mut request_bit_iter = nibble_iter.bits();
            let num_matched_bits =
                skip_common_prefix(&mut extension_bit_iter, &mut request_bit_iter);
            assert!(num_matched_bits < 4);
            // Note: We have to skip 1 bit here to ensure the right result. For example, assume the
            // extension node has 2 nibbles (8 bits) and only the first 5 bits are matched. The
            // siblings of the queried key should include 5 default hashes followed by `#1`, which
            // is the result of iteratively hashing `n` times from bottom up starting with `child`
            // where `n` equals the number of bits left after matching minus 1.
            //
            //```text
            //
            //                   #(final hash)
            //                  / \------------------> 1st bit \
            //                 #   placeholder                  \
            //                / \--------------------> 2nd bit   \
            //               #   placeholder                      } 1st nibble
            //              / \----------------------> 3rd bit   /
            //    placeholder  #                                /
            //                / \--------------------> 4th bit /
            //               #   placeholder
            //              / \----------------------> 5th bit \
            //    placeholder  \                                \
            //                / \--------------------> 6th bit   \
            //               #1  the queried key                  } 2nd nibble
            //              / \----------------------> 7th bit   /
            //             #   placeholder                      /
            //            / \------------------------> 8th bit /
            //       child   placeholder
            // ```
            extension_bit_iter.next();
            siblings.append(&mut vec![*SPARSE_MERKLE_PLACEHOLDER_HASH; num_matched_bits]);
            siblings.push(extension_bit_iter.rev().fold(self.child, |hash, bit| {
                if bit {
                    SparseMerkleInternalNode::new(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash).hash()
                } else {
                    SparseMerkleInternalNode::new(hash, *SPARSE_MERKLE_PLACEHOLDER_HASH).hash()
                }
            }));
            (siblings, true /* early termination */)
        } else {
            // 2. All the nibbles of the extension node match the nibble path of the queried key.
            // Then just return the siblings and a `false` telling the callsite to continue to
            // traverse the tree.
            (siblings, false /* early termination */)
        }
    }
}

impl LeafNode {
    /// Creates a new leaf node.
    pub fn new(key: HashValue, value_hash: HashValue) -> Self {
        Self { key, value_hash }
    }

    /// Gets the `key`.
    pub fn key(&self) -> HashValue {
        self.key
    }

    /// Gets the associated `value_hash`.
    pub fn value_hash(&self) -> HashValue {
        self.value_hash
    }

    /// Sets the associated `value_hash`.
    pub fn set_value_hash(&mut self, value_hash: HashValue) {
        self.value_hash = value_hash;
    }
}

impl Node {
    /// Creates the [`Branch`](Node::Branch) variant.
    pub fn new_branch(children: HashMap<u8, (HashValue, bool)>) -> Self {
        Node::Branch(BranchNode::new(children))
    }

    /// Creates the [`Extension`](Node::Extension) variant.
    pub fn new_extension(nibble_path: NibblePath, child: HashValue) -> Self {
        Node::Extension(ExtensionNode::new(nibble_path, child))
    }

    /// Creates the [`Leaf`](Node::Leaf) variant.
    pub fn new_leaf(key: HashValue, value_hash: HashValue) -> Self {
        Node::Leaf(LeafNode::new(key, value_hash))
    }

    /// Serializes to bytes for physical storage.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = vec![];
        match self {
            Node::Branch(branch_node) => {
                out.push(BranchNode::TAG);
                out.extend(serialize(&branch_node)?);
            }
            Node::Leaf(leaf_node) => {
                out.push(LeafNode::TAG);
                out.extend(serialize(leaf_node)?);
            }
            Node::Extension(extension_node) => {
                out.push(ExtensionNode::TAG);
                out.extend(serialize(extension_node)?);
            }
        }
        Ok(out)
    }

    /// Hashes are used to lookup the node in the database.
    pub fn hash(&self) -> HashValue {
        match self {
            Node::Branch(branch_node) => branch_node.hash(),
            Node::Extension(extension_node) => extension_node.hash(),
            Node::Leaf(leaf_node) => leaf_node.hash(),
        }
    }

    /// Recovers from serialized bytes in physical storage.
    pub fn decode(val: &[u8]) -> Result<Node> {
        if val.is_empty() {
            return Err(NodeDecodeError::EmptyInput.into());
        }
        let node_tag = val[0];
        match node_tag {
            BranchNode::TAG => Ok(Node::Branch(deserialize(&val[1..].to_vec())?)),
            ExtensionNode::TAG => Ok(Node::Extension(deserialize(&val[1..].to_vec())?)),
            LeafNode::TAG => Ok(Node::Leaf(deserialize(&val[1..].to_vec())?)),
            unknown_tag => Err(NodeDecodeError::UnknownTag { unknown_tag }.into()),
        }
    }
}

/// Error thrown when a [`Node`] fails to be deserialized out of a byte sequence stored in physical
/// storage, via [`Node::decode`].
#[derive(Debug, Fail, Eq, PartialEq)]
pub enum NodeDecodeError {
    /// Input is empty.
    #[fail(display = "Missing tag due to empty input")]
    EmptyInput,

    /// The first byte of the input is not a known tag representing one of the variants.
    #[fail(display = "lead tag byte is unknown: {}", unknown_tag)]
    UnknownTag { unknown_tag: u8 },
}
