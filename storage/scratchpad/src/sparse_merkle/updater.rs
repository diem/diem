// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    sparse_merkle::{
        node::{InternalNode, Node, NodeHandle},
        utils::{partition, swap_if, Either},
        UpdateError,
    },
    ProofRead,
};
use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_types::proof::{SparseMerkleLeafNode, SparseMerkleProof};
use std::{borrow::Borrow, cmp::Ordering};

type Result<T> = std::result::Result<T, UpdateError>;

type InMemSubTree<V> = super::node::SubTree<V>;
type InMemInternal<V> = super::node::InternalNode<V>;

#[derive(Clone)]
enum InMemSubTreeInfo<V> {
    Internal {
        subtree: InMemSubTree<V>,
        // TODO(aldenhu): make it lazy (going up)
        node: InMemInternal<V>,
    },
    Leaf {
        subtree: InMemSubTree<V>,
        key: HashValue,
    },
    Unknown {
        subtree: InMemSubTree<V>,
    },
    Empty,
}

impl<V: Clone + CryptoHash> InMemSubTreeInfo<V> {
    fn create_leaf_with_update(update: (HashValue, &V)) -> Self {
        let subtree = InMemSubTree::new_leaf_with_value(update.0, (*update.1).clone());
        Self::Leaf {
            key: update.0,
            subtree,
        }
    }

    fn create_leaf_with_proof(leaf: &SparseMerkleLeafNode) -> Self {
        let subtree = InMemSubTree::new_leaf_with_value_hash(leaf.key(), leaf.value_hash());
        Self::Leaf {
            key: leaf.key(),
            subtree,
        }
    }

    fn create_internal(left: Self, right: Self) -> Self {
        let node = InternalNode {
            left: left.into_subtree(),
            right: right.into_subtree(),
        };
        let subtree = InMemSubTree::NonEmpty {
            hash: node.calc_hash(),
            root: NodeHandle::new_shared(Node::Internal(node.clone())),
        };

        Self::Internal { subtree, node }
    }

    fn create_unknown(hash: HashValue) -> Self {
        Self::Unknown {
            subtree: InMemSubTree::new_unknown(hash),
        }
    }

    fn into_subtree(self) -> InMemSubTree<V> {
        match self {
            Self::Leaf { subtree, .. } => subtree,
            Self::Internal { subtree, .. } => subtree,
            Self::Unknown { subtree } => subtree,
            Self::Empty => InMemSubTree::Empty,
        }
    }

    fn combine(left: Self, right: Self) -> Self {
        // If there's a only leaf in the subtree,
        // rollup the leaf, otherwise create an internal node.
        match (&left, &right) {
            (Self::Empty, Self::Leaf { .. }) => right,
            (Self::Leaf { .. }, Self::Empty) => left,
            (Self::Empty, Self::Empty) => unreachable!(),
            _ => InMemSubTreeInfo::create_internal(left, right),
        }
    }
}

#[derive(Clone)]
enum PersistedSubTreeInfo<'a, V> {
    ProofPathInternal { proof: &'a SparseMerkleProof<V> },
    ProofSibling { hash: HashValue },
    Leaf { leaf: SparseMerkleLeafNode },
}

#[derive(Clone)]
enum SubTreeInfo<'a, V> {
    InMem(InMemSubTreeInfo<V>),
    Persisted(PersistedSubTreeInfo<'a, V>),
}

impl<'a, V: Clone + CryptoHash> SubTreeInfo<'a, V> {
    fn new_empty() -> Self {
        Self::InMem(InMemSubTreeInfo::Empty)
    }

    fn new_proof_leaf(leaf: SparseMerkleLeafNode) -> Self {
        Self::Persisted(PersistedSubTreeInfo::Leaf { leaf })
    }

    fn new_proof_sibling(hash: HashValue) -> Self {
        if hash == *SPARSE_MERKLE_PLACEHOLDER_HASH {
            Self::InMem(InMemSubTreeInfo::Empty)
        } else {
            Self::Persisted(PersistedSubTreeInfo::ProofSibling { hash })
        }
    }

    fn new_on_proof_path(proof: &'a SparseMerkleProof<V>, depth: usize) -> Self {
        match proof.siblings().len().cmp(&depth) {
            Ordering::Greater => Self::Persisted(PersistedSubTreeInfo::ProofPathInternal { proof }),
            Ordering::Equal => match proof.leaf() {
                Some(leaf) => Self::new_proof_leaf(leaf),
                None => Self::new_empty(),
            },
            _ => unreachable!(),
        }
    }

    fn from_persisted(
        a_descendant_key: HashValue,
        depth: usize,
        proof_reader: &'a impl ProofRead<V>,
    ) -> Result<Self> {
        let proof = proof_reader
            .get_proof(a_descendant_key)
            .ok_or(UpdateError::MissingProof)?;
        if depth > proof.siblings().len() {
            return Err(UpdateError::ShortProof {
                key: a_descendant_key,
                num_siblings: proof.siblings().len(),
                depth,
            });
        }
        Ok(Self::new_on_proof_path(proof, depth))
    }

    fn from_in_mem(subtree: &InMemSubTree<V>) -> Self {
        match &subtree {
            InMemSubTree::Empty => SubTreeInfo::new_empty(),
            InMemSubTree::NonEmpty { root, .. } => match root.get_node_if_in_mem() {
                Some(arc_node) => match arc_node.borrow() {
                    Node::Internal(node) => SubTreeInfo::InMem(InMemSubTreeInfo::Internal {
                        node: (*node).clone(),
                        subtree: subtree.weak(),
                    }),
                    Node::Leaf(node) => SubTreeInfo::InMem(InMemSubTreeInfo::Leaf {
                        key: node.key,
                        subtree: subtree.weak(),
                    }),
                },
                None => SubTreeInfo::InMem(InMemSubTreeInfo::Unknown {
                    subtree: subtree.weak(),
                }),
            },
        }
    }

    fn is_unknown(&self) -> bool {
        matches!(self, Self::InMem(InMemSubTreeInfo::Unknown { .. }))
            || matches!(
                self,
                Self::Persisted(PersistedSubTreeInfo::ProofSibling { .. })
            )
    }

    fn into_children(
        self,
        a_descendent_key: HashValue,
        depth: usize,
        proof_reader: &'a impl ProofRead<V>,
    ) -> Result<(Self, Self)> {
        let myself = if self.is_unknown() {
            SubTreeInfo::from_persisted(a_descendent_key, depth, proof_reader)?
        } else {
            self
        };

        Ok(match &myself {
            SubTreeInfo::InMem(info) => match info {
                InMemSubTreeInfo::Empty => (Self::new_empty(), Self::new_empty()),
                InMemSubTreeInfo::Leaf { key, .. } => {
                    let key = *key;
                    swap_if(myself, SubTreeInfo::new_empty(), key.bit(depth))
                }
                InMemSubTreeInfo::Internal { node, .. } => (
                    SubTreeInfo::from_in_mem(&node.left),
                    SubTreeInfo::from_in_mem(&node.right),
                ),
                InMemSubTreeInfo::Unknown { .. } => unreachable!(),
            },
            SubTreeInfo::Persisted(info) => match info {
                PersistedSubTreeInfo::Leaf { leaf } => {
                    let key = leaf.key();
                    swap_if(myself, SubTreeInfo::new_empty(), key.bit(depth))
                }
                PersistedSubTreeInfo::ProofPathInternal { proof } => {
                    let siblings = proof.siblings();
                    assert!(siblings.len() > depth);
                    let sibling_child =
                        SubTreeInfo::new_proof_sibling(siblings[siblings.len() - depth - 1]);
                    let on_path_child = SubTreeInfo::new_on_proof_path(proof, depth + 1);
                    swap_if(on_path_child, sibling_child, a_descendent_key.bit(depth))
                }
                PersistedSubTreeInfo::ProofSibling { .. } => unreachable!(),
            },
        })
    }

    fn materialize(self) -> InMemSubTreeInfo<V> {
        match self {
            Self::InMem(info) => info,
            Self::Persisted(info) => match info {
                PersistedSubTreeInfo::Leaf { leaf } => {
                    InMemSubTreeInfo::create_leaf_with_proof(&leaf)
                }
                PersistedSubTreeInfo::ProofSibling { hash } => {
                    InMemSubTreeInfo::create_unknown(hash)
                }
                PersistedSubTreeInfo::ProofPathInternal { .. } => {
                    unreachable!()
                }
            },
        }
    }
}

pub struct SubTreeUpdater<'a, V> {
    depth: usize,
    info: SubTreeInfo<'a, V>,
    updates: &'a [(HashValue, &'a V)],
}

impl<'a, V: Send + Sync + Clone + CryptoHash> SubTreeUpdater<'a, V> {
    pub(crate) fn update(
        root: InMemSubTree<V>,
        updates: &'a [(HashValue, &'a V)],
        proof_reader: &'a impl ProofRead<V>,
    ) -> Result<InMemSubTree<V>> {
        let updater = Self {
            depth: 0,
            info: SubTreeInfo::from_in_mem(&root),
            updates,
        };
        Ok(updater.run(proof_reader)?.into_subtree())
    }

    fn run(self, proof_reader: &impl ProofRead<V>) -> Result<InMemSubTreeInfo<V>> {
        match self.maybe_end_recursion() {
            Either::A(ended) => Ok(ended),
            Either::B(myself) => {
                let (left, right) = myself.into_children(proof_reader)?;
                let (left_ret, right_ret) =
                    rayon::join(|| left.run(proof_reader), || right.run(proof_reader));

                Ok(InMemSubTreeInfo::combine(left_ret?, right_ret?))
            }
        }
    }

    fn maybe_end_recursion(self) -> Either<InMemSubTreeInfo<V>, Self> {
        match self.updates.len() {
            0 => Either::A(self.info.materialize()),
            1 => match &self.info {
                SubTreeInfo::InMem(in_mem_info) => match in_mem_info {
                    InMemSubTreeInfo::Empty => {
                        Either::A(InMemSubTreeInfo::create_leaf_with_update(self.updates[0]))
                    }
                    InMemSubTreeInfo::Leaf { key, .. } => Either::or(
                        *key == self.updates[0].0,
                        InMemSubTreeInfo::create_leaf_with_update(self.updates[0]),
                        self,
                    ),
                    _ => Either::B(self),
                },
                SubTreeInfo::Persisted(PersistedSubTreeInfo::Leaf { leaf }) => Either::or(
                    leaf.key() == self.updates[0].0,
                    InMemSubTreeInfo::create_leaf_with_update(self.updates[0]),
                    self,
                ),
                _ => Either::B(self),
            },
            _ => Either::B(self),
        }
    }

    fn into_children(self, proof_reader: &'a impl ProofRead<V>) -> Result<(Self, Self)> {
        let pivot = partition(self.updates, self.depth);
        let (left_updates, right_updates) = self.updates.split_at(pivot);
        let (left_info, right_info) =
            self.info
                .into_children(self.updates[0].0, self.depth, proof_reader)?;

        Ok((
            Self {
                depth: self.depth + 1,
                info: left_info,
                updates: left_updates,
            },
            Self {
                depth: self.depth + 1,
                info: right_info,
                updates: right_updates,
            },
        ))
    }
}
