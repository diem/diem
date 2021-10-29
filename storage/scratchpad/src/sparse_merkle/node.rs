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

#[derive(Clone, Debug)]
pub(crate) struct InternalNode<V> {
    pub left: SubTree<V>,
    pub right: SubTree<V>,
}

impl<V: CryptoHash> InternalNode<V> {
    pub fn calc_hash(&self) -> HashValue {
        SparseMerkleInternalNode::new(self.left.hash(), self.right.hash()).hash()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LeafNode<V> {
    pub key: HashValue,
    pub value: LeafValue<V>,
}

impl<V> LeafNode<V> {
    pub fn new(key: HashValue, value: LeafValue<V>) -> Self {
        Self { key, value }
    }

    pub fn clone_with_weak_value(&self) -> Self {
        Self {
            key: self.key,
            value: self.value.weak(),
        }
    }
}

impl<V: CryptoHash> LeafNode<V> {
    pub fn calc_hash(&self) -> HashValue {
        SparseMerkleLeafNode::new(self.key, self.value.hash).hash()
    }
}

impl<V> From<&SparseMerkleLeafNode> for LeafNode<V>
where
    V: CryptoHash,
{
    fn from(leaf_node: &SparseMerkleLeafNode) -> Self {
        Self {
            key: leaf_node.key(),
            value: LeafValue::new_with_value_hash(leaf_node.value_hash()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NodeInner<V> {
    Internal(InternalNode<V>),
    Leaf(LeafNode<V>),
}

#[derive(Debug)]
pub(crate) struct Node<V> {
    generation: u64,
    inner: NodeInner<V>,
}

impl<V: CryptoHash> Node<V> {
    pub fn calc_hash(&self) -> HashValue {
        match &self.inner {
            NodeInner::Internal(internal_node) => internal_node.calc_hash(),
            NodeInner::Leaf(leaf_node) => leaf_node.calc_hash(),
        }
    }
}

impl<V> Node<V> {
    pub fn new_leaf(key: HashValue, value: LeafValue<V>, generation: u64) -> Self {
        Self {
            generation,
            inner: NodeInner::Leaf(LeafNode::new(key, value)),
        }
    }

    pub fn new_leaf_from_node(node: LeafNode<V>, generation: u64) -> Self {
        Self {
            generation,
            inner: NodeInner::Leaf(node),
        }
    }

    pub fn new_internal(left: SubTree<V>, right: SubTree<V>, generation: u64) -> Self {
        Self {
            generation,
            inner: NodeInner::Internal(InternalNode { left, right }),
        }
    }

    pub fn new_internal_from_node(node: InternalNode<V>, generation: u64) -> Self {
        Self {
            generation,
            inner: NodeInner::Internal(node),
        }
    }

    pub fn inner(&self) -> &NodeInner<V> {
        &self.inner
    }
}

#[derive(Debug)]
pub enum Ref<R> {
    Shared(Arc<R>),
    Weak(Weak<R>),
}

impl<R> Ref<R> {
    pub fn new_unknown() -> Self {
        Self::Weak(Weak::new())
    }

    pub fn new_shared(referee: R) -> Self {
        Self::Shared(Arc::new(referee))
    }

    pub fn weak(&self) -> Self {
        Self::Weak(match self {
            Self::Shared(arc) => Arc::downgrade(arc),
            Self::Weak(weak) => weak.clone(),
        })
    }

    pub fn get_if_in_mem(&self) -> Option<Arc<R>> {
        match self {
            Self::Shared(arc) => Some(arc.clone()),
            Self::Weak(weak) => weak.upgrade(),
        }
    }
}

impl<R> Clone for Ref<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Shared(arc) => Self::Shared(arc.clone()),
            Self::Weak(weak) => Self::Weak(weak.clone()),
        }
    }
}

pub(crate) type NodeHandle<V> = Ref<Node<V>>;

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
            root: NodeHandle::new_unknown(),
        }
    }

    pub fn new_leaf_with_value(key: HashValue, value: V, generation: u64) -> Self {
        Self::new_leaf_impl(key, LeafValue::new_with_value(value), generation)
    }

    pub fn new_leaf_with_value_hash(
        key: HashValue,
        value_hash: HashValue,
        generation: u64,
    ) -> Self {
        Self::new_leaf_impl(key, LeafValue::new_with_value_hash(value_hash), generation)
    }

    fn new_leaf_impl(key: HashValue, value: LeafValue<V>, generation: u64) -> Self {
        let leaf = Node::new_leaf(key, value, generation);

        Self::NonEmpty {
            hash: leaf.calc_hash(),
            root: NodeHandle::new_shared(leaf),
        }
    }

    pub fn new_internal(left: Self, right: Self, generation: u64) -> Self {
        let internal = Node::new_internal(left, right, generation);

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
            Self::NonEmpty { root, .. } => root.get_if_in_mem(),
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

#[derive(Clone, Debug)]
pub struct LeafValue<V> {
    pub hash: HashValue,
    pub data: Ref<V>,
}

impl<V> LeafValue<V> {
    pub fn new_with_value(value: V) -> Self
    where
        V: CryptoHash,
    {
        Self {
            hash: value.hash(),
            data: Ref::new_shared(value),
        }
    }

    pub fn new_with_value_hash(value_hash: HashValue) -> Self {
        Self {
            hash: value_hash,
            data: Ref::new_unknown(),
        }
    }

    pub fn weak(&self) -> Self {
        Self {
            hash: self.hash,
            data: self.data.weak(),
        }
    }
}
