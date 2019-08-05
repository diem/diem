// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Node types of [`JellyfishMerkleTree`](crate::JellyfishMerkleTree)
//!
//! This module defines two types of Jellyfish Merkle tree nodes: [`InternalNode`]
//! and [`LeafNode`] as building blocks of a 256-bit
//! [`JellyfishMerkleTree`](crate::JellyfishMerkleTree). [`InternalNode`] represents a 4-level
//! binary tree to optimize for IOPS: it compresses a tree with 31 nodes into one node with 16
//! chidren at the lowest level. [`LeafNode`] stores the full key and the account blob data
//! associated.

#[cfg(test)]
mod node_type_test;

use bincode::{deserialize, serialize};
use crypto::{
    hash::{
        CryptoHash, SparseMerkleInternalHasher, SparseMerkleLeafHasher,
        SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
use failure::{Fail, Result};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};
use sparse_merkle::nibble_path::NibblePath;
use std::collections::hash_map::HashMap;
use types::{
    account_state_blob::AccountStateBlob,
    proof::{SparseMerkleInternalNode, SparseMerkleLeafNode},
    transaction::Version,
};

/// The unique key of each node.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NodeKey {
    // The version at which the node is created.
    version: Version,
    // The nibble path this node represents in the tree.
    nibble_path: NibblePath,
}

impl NodeKey {
    /// Creates a new `NodeKey`.
    pub fn new(version: Version, nibble_path: NibblePath) -> Self {
        Self {
            version,
            nibble_path,
        }
    }

    /// A shortcut to generate a node key consisting of a version and an empty nibble path.
    pub fn new_empty_path(version: Version) -> Self {
        Self::new(version, NibblePath::new(vec![]))
    }

    /// Gets the version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Gets the nibble path.
    pub fn nibble_path(&self) -> &NibblePath {
        &self.nibble_path
    }

    /// Generates a child node_key based on this node key.
    pub fn gen_child_node_key(&self, version: Version, n: u8) -> Self {
        let mut node_nibble_path = self.nibble_path().clone();
        node_nibble_path.push(n);
        Self::new(version, node_nibble_path)
    }

    /// Sets the version to the given version.
    pub fn set_version(&mut self, version: Version) {
        self.version = version;
    }
}

/// Each child of [`InternalNode`] encapsulates a nibble forking at this node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Child {
    // The hash value of this child node.
    pub hash: HashValue,
    // `version`, the `nibble_path` of the ['NodeKey`] of this [`InternalNode`] the child belongs
    // to and the child's index constitute the [`NodeKey`] to uniquely identify this child node
    // from the storage. Used by `[`NodeKey::gen_child_node_key`].
    pub version: Version,
    // Whether the child is a leaf node.
    pub is_leaf: bool,
}

impl Child {
    pub fn new(hash: HashValue, version: Version, is_leaf: bool) -> Self {
        Self {
            hash,
            version,
            is_leaf,
        }
    }
}

/// [`Children`] is just a collection of children belonging to a [`InternalNode`], indexed from 0 to
/// 15, inclusive.
pub(crate) type Children = HashMap<u8, Child>;

/// Represents a 4-level subtree with 16 children at the bottom level. Theoretically, this reduces
/// IOPS to query a tree by 4x since we compress 4 levels in a standard Merkle tree into 1 node.
/// Though we choose the same internal node structure as that of Patricia Merkle tree, the root hash
/// computation logic is similar to a 4-level sparse Merkle tree except for some customizations. See
/// the `CryptoHash` trait implementation below for details.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InternalNode {
    // Up to 16 children.
    children: Children,
}

/// Represents an account.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LeafNode {
    // The hashed account address associated with this leaf node.
    account_key: HashValue,
    // The hash of the account state blob.
    blob_hash: HashValue,
    // The account blob associated with `account_key`.
    blob: AccountStateBlob,
}

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
enum NodeTag {
    Internal = 1,
    Leaf = 2,
}

/// The explicit tag is used as a prefix in the encoded format of nodes to distinguish different
/// node discrinminants.
trait Tag {
    const TAG: NodeTag;
}

impl Tag for InternalNode {
    const TAG: NodeTag = NodeTag::Internal;
}

impl Tag for LeafNode {
    const TAG: NodeTag = NodeTag::Leaf;
}

/// Computes the hash of internal node according to [`JellyfishTree`](crate::JellyfishTree)
/// data structure in the logical view. `start` and `nibble_height` determine a subtree whose
/// root hash we want to get. For an internal node with 16 children at the bottom level, we compute
/// the root hash of it as if a full binary Merkle tree with 16 leaves as below:
///
/// ```text
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
/// parent with the hash of its sibling. Finally, we can get the hash of this internal node.
///
/// However, if an internal node doesn't have all 16 chidren exist at height 0 but just a few of
/// them, we have a modified hashing rule on top of what is stated above:
/// 1. From top to bottom, a node will be replaced by a leaf child if the subtree rooted at this
/// node has only one child at height 0 and it is a leaf child.
/// 2. From top to bottom, a node will be replaced by the placeholder node if the subtree rooted at
/// this node doesn't have any child at height 0. For example, if an internal node has 3 leaf
/// children at index 0, 3, 8, respectively, and 1 internal node at index C, then the computation
/// graph will be like:
///
/// ```text
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
impl CryptoHash for InternalNode {
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

/// Computes the hash of a [`LeafNode`].
impl CryptoHash for LeafNode {
    // Unused hasher.
    type Hasher = SparseMerkleLeafHasher;

    fn hash(&self) -> HashValue {
        SparseMerkleLeafNode::new(self.account_key, self.blob_hash).hash()
    }
}

/// The concrete node type of [`JellyfishMerkleTree`](crate::JellyfishMerkleTree).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Node {
    /// A wrapper of [`InternalNode`].
    Internal(InternalNode),
    /// A wrapper of [`LeafNode`].
    Leaf(LeafNode),
}

impl From<InternalNode> for Node {
    fn from(node: InternalNode) -> Self {
        Node::Internal(node)
    }
}

impl From<LeafNode> for Node {
    fn from(node: LeafNode) -> Self {
        Node::Leaf(node)
    }
}

impl InternalNode {
    /// Creates a new Internal node.
    pub fn new(children: Children) -> Self {
        Self { children }
    }

    /// Sets the `n`-th child.
    pub fn set_child(&mut self, n: u8, child: Child) {
        assert!(n < 16);
        self.children.insert(n, child);
    }

    /// Gets the `n`-th child.
    pub fn child(&self, n: u8) -> Option<&Child> {
        self.children.get(&n)
    }

    /// Return the total number of existing children.
    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    /// Generates `existence_bitmap` and `leaf_bitmap` as a pair of `u16`s: child at index `i`
    /// exists if `existence_bitmap[i]` is set; child at index `i` is leaf node if
    /// `leaf_bitmap[i]` is set.
    pub fn generate_bitmaps(&self) -> (u16, u16) {
        let mut existence_bitmap = 0;
        let mut leaf_bitmap = 0;
        for (i, child) in self.children.iter() {
            existence_bitmap |= 1u16 << i;
            leaf_bitmap |= (child.is_leaf as u16) << i;
        }
        // `leaf_bitmap` must be a subset of `existence_bitmap`.
        assert_eq!(existence_bitmap | leaf_bitmap, existence_bitmap);
        (existence_bitmap, leaf_bitmap)
    }

    /// Given a range [start, start + width), returns the sub-bitmap of that range.
    fn range_bitmaps(start: u8, width: u8, bitmaps: (u16, u16)) -> (u16, u16) {
        assert!(start < 16 && width.count_ones() == 1 && start % width == 0);
        // A range with `start == 8` and `width == 4` will generate a mask 0b0000111100000000.
        let mask = if width == 16 {
            0xffff
        } else {
            assert!(width <= 16);
            (1 << width) - 1
        } << start;
        (bitmaps.0 & mask, bitmaps.1 & mask)
    }

    fn merkle_hash(
        &self,
        start: u8,
        width: u8,
        (existence_bitmap, leaf_bitmap): (u16, u16),
    ) -> HashValue {
        // Given a bit [start, 1 << nibble_height], return the value of that range.
        let (range_existence_bitmap, range_leaf_bitmap) =
            InternalNode::range_bitmaps(start, width, (existence_bitmap, leaf_bitmap));
        if range_existence_bitmap == 0 {
            // No child under this subtree
            *SPARSE_MERKLE_PLACEHOLDER_HASH
        } else if range_existence_bitmap.count_ones() == 1 && (range_leaf_bitmap != 0 || width == 1)
        {
            // Only 1 leaf child under this subtree or reach the lowest level
            let only_child_index = range_existence_bitmap.trailing_zeros() as u8;
            self.child(only_child_index)
                .unwrap_or_else(|| {
                    panic!(
                        "Corrupted internal node: existence_bitmap indicates \
                         the existence of a non-exist child at index {}",
                        only_child_index
                    )
                })
                .hash
        } else {
            let left_child = self.merkle_hash(start, width / 2, (existence_bitmap, leaf_bitmap));
            let right_child = self.merkle_hash(
                start + width / 2,
                width / 2,
                (existence_bitmap, leaf_bitmap),
            );
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
    pub fn get_child_with_siblings(
        &self,
        node_key: &NodeKey,
        n: u8,
    ) -> (Option<NodeKey>, Vec<HashValue>) {
        let mut siblings = vec![];
        assert!(n < 16);
        let (existence_bitmap, leaf_bitmap) = self.generate_bitmaps();

        // Nibble height from 3 to 0.
        for h in (0..4).rev() {
            // Get the number of children of the internal node that each subtree at this height
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
            siblings.push(self.merkle_hash(
                sibling_half_start,
                width,
                (existence_bitmap, leaf_bitmap),
            ));

            let (range_existence_bitmap, range_leaf_bitmap) = InternalNode::range_bitmaps(
                child_half_start,
                width,
                (existence_bitmap, leaf_bitmap),
            );

            if range_existence_bitmap == 0 {
                // No child in this range.
                return (None, siblings);
            } else if range_existence_bitmap.count_ones() == 1
                && (range_leaf_bitmap.count_ones() == 1 || width == 1)
            {
                // Return the only 1 leaf child under this subtree or reach the lowest level
                // Even this leaf child is not the n-th child, it should be returned instead of
                // `None` because it's existence indirectly proves the n-th child doesn't exist.
                // Please read proof format for details.
                let only_child_index = range_existence_bitmap.trailing_zeros() as u8;
                return (
                    {
                        let only_child_version = self
                            .child(only_child_index)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Corrupted internal node: child_bitmap indicates \
                                     the existence of a non-exist child at index {}",
                                    only_child_index
                                )
                            })
                            .version;
                        Some(node_key.gen_child_node_key(only_child_version, only_child_index))
                    },
                    siblings,
                );
            }
        }
        unreachable!("Impossible to get here without returning even at the lowest level.")
    }
}

impl LeafNode {
    /// Creates a new leaf node.
    pub fn new(account_key: HashValue, blob: AccountStateBlob) -> Self {
        let blob_hash = blob.hash();
        Self {
            account_key,
            blob_hash,
            blob,
        }
    }

    /// Gets the account key, the hashed account address.
    pub fn account_key(&self) -> HashValue {
        self.account_key
    }

    /// Gets the hash of associated blob.
    pub fn blob_hash(&self) -> HashValue {
        self.blob_hash
    }

    /// Gets the associated blob itself.
    pub fn blob(&self) -> &AccountStateBlob {
        &self.blob
    }
}

impl Node {
    /// Creates the [`Internal`](Node::Internal) variant.
    pub fn new_internal(children: HashMap<u8, Child>) -> Self {
        Node::Internal(InternalNode::new(children))
    }

    /// Creates the [`Leaf`](Node::Leaf) variant.
    pub fn new_leaf(account_key: HashValue, blob: AccountStateBlob) -> Self {
        Node::Leaf(LeafNode::new(account_key, blob))
    }

    /// Returns `true` if the node is a leaf node.
    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf(_) => true,
            _ => false,
        }
    }

    /// Serializes to bytes for physical storage.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = vec![];
        match self {
            Node::Internal(internal_node) => {
                out.push(InternalNode::TAG as u8);
                out.extend(serialize(&internal_node)?);
            }
            Node::Leaf(leaf_node) => {
                out.push(LeafNode::TAG as u8);
                out.extend(serialize(&leaf_node)?);
            }
        }
        Ok(out)
    }

    /// Computes the hash of nodes.
    pub fn hash(&self) -> HashValue {
        match self {
            Node::Internal(internal_node) => internal_node.hash(),
            Node::Leaf(leaf_node) => leaf_node.hash(),
        }
    }

    /// Recovers from serialized bytes in physical storage.
    pub fn decode(val: &[u8]) -> Result<Node> {
        if val.is_empty() {
            return Err(NodeDecodeError::EmptyInput.into());
        }
        let tag = val[0];
        let node_tag = NodeTag::from_u8(tag);
        match node_tag {
            Some(NodeTag::Internal) => Ok(Node::Internal(deserialize(&val[1..])?)),
            Some(NodeTag::Leaf) => Ok(Node::Leaf(deserialize(&val[1..])?)),
            None => Err(NodeDecodeError::UnknownTag { unknown_tag: tag }.into()),
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
