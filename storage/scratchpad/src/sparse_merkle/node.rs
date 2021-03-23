// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines all kinds of structures in the Sparse Merkle Tree maintained in scratch pad.
//! There are four kinds of nodes:
//! - A `SubTree::Empty` represents an empty subtree with zero leaf. Its root hash is assumed to be
//! the default hash.
//!
//! - A `SubTree::NonEmpty` represents a subtree with one or more leaves, it carries its root hash.
//!
//! From a `SubTree::NonEmpty` one may or may not get an reference to its root node, depending on
//! how this subtree structure was created and if the root node has been dropped (when its persisted
//! to DB and given up by any possible cache). A non empty subtree can refer to one of two types of
//! nodes as its root:
//!
//! - An `InternalNode` is a node that has two children. It is same as the internal node in a
//! standard Merkle tree.
//!
//! - A `LeafNode` represents a single account. Similar to what is in storage, a leaf node has a
//! key which is the hash of the account address as well as a value hash which is the hash of the
//! corresponding account content. The difference is that a `LeafNode` does not always have the
//! value, in the case when the leaf was loaded into memory as part of a non-inclusion proof.

use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_types::proof::{SparseMerkleInternalNode, SparseMerkleLeafNode};
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub(crate) struct InternalNode<V> {
    pub left: SubTree<V>,
    pub right: SubTree<V>,
}

impl<V: CryptoHash> InternalNode<V> {
    pub fn calc_hash(&self) -> HashValue {
        SparseMerkleInternalNode::new(self.left.hash(), self.right.hash()).hash()
    }
}

#[derive(Debug)]
pub(crate) struct LeafNode<V> {
    pub key: HashValue,
    pub value: LeafValue<V>,
}

impl<V: CryptoHash> LeafNode<V> {
    pub fn new(key: HashValue, value: LeafValue<V>) -> Self {
        Self { key, value }
    }
    pub fn calc_hash(&self) -> HashValue {
        SparseMerkleLeafNode::new(self.key, self.value.calc_hash()).hash()
    }
}

impl<V> From<SparseMerkleLeafNode> for LeafNode<V>
where
    V: CryptoHash,
{
    fn from(leaf_node: SparseMerkleLeafNode) -> Self {
        Self {
            key: leaf_node.key(),
            value: LeafValue::ValueHash(leaf_node.value_hash()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Node<V> {
    Internal(InternalNode<V>),
    Leaf(LeafNode<V>),
}

impl<V: CryptoHash> Node<V> {
    pub fn new_leaf(key: HashValue, value: LeafValue<V>) -> Self {
        Self::Leaf(LeafNode::new(key, value))
    }

    pub fn new_internal(left: SubTree<V>, right: SubTree<V>) -> Self {
        Self::Internal(InternalNode { left, right })
    }

    pub fn calc_hash(&self) -> HashValue {
        match self {
            Self::Internal(internal_node) => internal_node.calc_hash(),
            Self::Leaf(leaf_node) => leaf_node.calc_hash(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum NodeHandle<V> {
    Shared(Arc<Node<V>>),
    Weak(Weak<Node<V>>),
}

impl<V> NodeHandle<V> {
    pub fn new_unkonwn() -> Self {
        Self::Weak(Weak::new())
    }

    pub fn new_shared(node: Node<V>) -> Self {
        Self::Shared(Arc::new(node))
    }

    pub fn weak(&self) -> Self {
        Self::Weak(match self {
            Self::Shared(arc) => Arc::downgrade(arc),
            Self::Weak(weak) => weak.clone(),
        })
    }

    pub fn get_node_if_in_mem(&self) -> Option<Arc<Node<V>>> {
        match self {
            Self::Shared(arc) => Some(arc.clone()),
            Self::Weak(weak) => weak.upgrade(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SubTree<V> {
    Empty,
    NonEmpty {
        hash: HashValue,
        root: NodeHandle<V>,
    },
}

impl<V: CryptoHash> SubTree<V> {
    pub fn new_empty() -> Self {
        Self::Empty
    }

    pub fn new_unknown(hash: HashValue) -> Self {
        Self::NonEmpty {
            hash,
            root: NodeHandle::new_unkonwn(),
        }
    }

    pub fn new_leaf_with_value(key: HashValue, value: V) -> Self {
        Self::new_leaf_impl(key, LeafValue::Value(value))
    }

    pub fn new_leaf_with_value_hash(key: HashValue, value_hash: HashValue) -> Self {
        Self::new_leaf_impl(key, LeafValue::ValueHash(value_hash))
    }

    fn new_leaf_impl(key: HashValue, value: LeafValue<V>) -> Self {
        let leaf = Node::new_leaf(key, value);

        Self::NonEmpty {
            hash: leaf.calc_hash(),
            root: NodeHandle::new_shared(leaf),
        }
    }

    pub fn new_internal(left: Self, right: Self) -> Self {
        let internal = Node::new_internal(left, right);

        Self::NonEmpty {
            hash: internal.calc_hash(),
            root: NodeHandle::new_shared(internal),
        }
    }

    pub fn hash(&self) -> HashValue {
        match self {
            Self::Empty => *SPARSE_MERKLE_PLACEHOLDER_HASH,
            Self::NonEmpty { hash, .. } => *hash,
        }
    }

    pub fn weak(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::NonEmpty { hash, root } => Self::NonEmpty {
                hash: *hash,
                root: root.weak(),
            },
        }
    }

    pub fn get_node_if_in_mem(&self) -> Option<Arc<Node<V>>> {
        match self {
            Self::Empty => None,
            Self::NonEmpty { root, .. } => root.get_node_if_in_mem(),
        }
    }

    #[cfg(test)]
    pub fn is_unknown(&self) -> bool {
        matches!(
            self,
            Self::NonEmpty {
                root: NodeHandle::Weak(_),
                ..
            }
        )
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        matches!(self, SubTree::Empty)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeafValue<V> {
    /// The content of the leaf.
    Value(V),

    /// The hash of the leaf content.
    ValueHash(HashValue),
}

impl<V: CryptoHash> LeafValue<V> {
    pub fn calc_hash(&self) -> HashValue {
        match self {
            LeafValue::Value(val) => val.hash(),
            LeafValue::ValueHash(val_hash) => *val_hash,
        }
    }
}
