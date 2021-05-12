// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements an in-memory Sparse Merkle Tree that is similar to what we use in
//! storage to represent world state. This tree will store only a small portion of the state -- the
//! part of accounts that have been modified by uncommitted transactions. For example, if we
//! execute a transaction T_i on top of committed state and it modified account A, we will end up
//! having the following tree:
//! ```text
//!              S_i
//!             /   \
//!            o     y
//!           / \
//!          x   A
//! ```
//! where A has the new state of the account, and y and x are the siblings on the path from root to
//! A in the tree.
//!
//! This Sparse Merkle Tree is immutable once constructed. If the next transaction T_{i+1} modified
//! another account B that lives in the subtree at y, a new tree will be constructed and the
//! structure will look like the following:
//! ```text
//!                 S_i        S_{i+1}
//!                /   \      /       \
//!               /     y   /          \
//!              / _______/             \
//!             //                       \
//!            o                          y'
//!           / \                        / \
//!          x   A                      z   B
//! ```
//!
//! Using this structure, we are able to query the global state, taking into account the output of
//! uncommitted transactions. For example, if we want to execute another transaction T_{i+1}', we
//! can use the tree S_i. If we look for account A, we can find its new value in the tree.
//! Otherwise, we know the account does not exist in the tree, and we can fall back to storage. As
//! another example, if we want to execute transaction T_{i+2}, we can use the tree S_{i+1} that
//! has updated values for both account A and B.
//!
//! Each version of the tree holds a strong reference (an Arc<Node>) to its root as well as one to
//! its base tree (S_i is the base tree of S_{i+1} in the above example). The root node in turn,
//! recursively holds all descendant nodes created in the same version, and weak references
//! (a Weak<Node>) to all descendant nodes that was created from previous versions.
//! With this construction:
//!     1. Even if a reference to a specific tree is dropped, the nodes belonging to it won't be
//! dropped as long as trees depending on it still hold strong references to it via the chain of
//! "base trees".
//!     2. Even if a tree is not dropped, when nodes it created are persisted to DB, all of them
//! and those created by its previous versions can be dropped, which we express by calling "prune()"
//! on it which replaces the strong references to its root and its base tree with weak references.
//!     3. We can hold strong references to recently accessed nodes that have already been persisted
//! in an LRU flavor cache for less DB reads.
//!
//! This Sparse Merkle Tree serves a dual purpose. First, to support a leader based consensus
//! algorithm, we need to build a tree of transactions like the following:
//! ```text
//! Committed -> T5 -> T6  -> T7
//!              └---> T6' -> T7'
//!                    └----> T7"
//! ```
//! Once T5 is executed, we will have a tree that stores the modified portion of the state. Later
//! when we execute T6 on top of T5, the output of T5 can be visible to T6.
//!
//! Second, given this tree representation it is straightforward to compute the root hash of S_i
//! once T_i is executed. This allows us to verify the proofs we need when executing T_{i+1}.

// See https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=e9c4c53eb80b30d09112fcfb07d481e7
#![allow(clippy::let_and_return)]
// See https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=795cd4f459f1d4a0005a99650726834b
#![allow(clippy::while_let_loop)]

mod node;
mod utils;

#[cfg(test)]
mod sparse_merkle_test;
#[cfg(test)]
mod test_utils;

use crate::sparse_merkle::{
    node::{LeafValue, Node, SubTree},
    utils::{partition, swap_if},
};
use arc_swap::{ArcSwap, ArcSwapOption};
use diem_crypto::{
    hash::{CryptoHash, HashValueBitIterator, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_types::proof::{SparseMerkleInternalNode, SparseMerkleLeafNode, SparseMerkleProof};
use std::{borrow::Borrow, cmp, collections::BTreeMap, sync::Arc};

/// `AccountStatus` describes the result of querying an account from this SparseMerkleTree.
#[derive(Debug, Eq, PartialEq)]
pub enum AccountStatus<V> {
    /// The account exists in the tree, therefore we can give its value.
    ExistsInScratchPad(V),

    /// The account does not exist in the tree, but exists in DB. This happens when the search
    /// reaches a leaf node that has the requested account, but the node has only the value hash
    /// because it was loaded into memory as part of a non-inclusion proof. When we go to DB we
    /// don't need to traverse the tree to find the same leaf, instead we can use the value hash to
    /// look up the account content directly.
    ExistsInDB,

    /// The account does not exist in either the tree or DB. This happens when the search reaches
    /// an empty node, or a leaf node that has a different account.
    DoesNotExist,

    /// We do not know if this account exists or not and need to go to DB to find out. This happens
    /// when the search reaches a subtree node.
    Unknown,
}

/// The inner content of a sparse merkle tree, we have this so that even if a tree is dropped, the
/// INNER of it can still live if referenced by a later version.
#[derive(Debug)]
struct Inner<V> {
    /// Reference to the root node, initially a strong reference, and once pruned, becomes a weak
    /// reference, allowing nodes created by this version to go away.
    root: ArcSwap<SubTree<V>>,
    /// Reference to the INNER base tree, needs to be a strong reference if the base is speculative
    /// itself, so that nodes referenced in this version won't go away because the base tree is
    /// dropped.
    base: ArcSwapOption<Inner<V>>,
}

impl<V: CryptoHash> Inner<V> {
    fn prune(&self) {
        // Replace the link to the root node with a weak reference, so all nodes created by this
        // version can be dropped. A weak link is still maintained so that if it's cached somehow,
        // we still have access to it without resorting to the DB.
        self.root.store(Arc::new(self.root.load().weak()));
        // Disconnect the base tree, so that nodes created by previous versions can be dropped.
        self.base.store(None);
    }
}

impl<V> Drop for Inner<V> {
    fn drop(&mut self) {
        let mut cur = self.base.swap(None);

        loop {
            if let Some(arc) = cur {
                if Arc::strong_count(&arc) == 1 {
                    // The only ref is the one we are now holding, so it'll be dropped after we free
                    // `arc`, which results in the chain of `base`s being dropped recursively,
                    // and that might trigger a stack overflow. To prevent that we follow the chain
                    // further to disconnect things beforehand.
                    cur = arc.base.swap(None);
                    continue;
                }
            }
            break;
        }
    }
}

/// The Sparse Merkle Tree implementation.
#[derive(Clone, Debug)]
pub struct SparseMerkleTree<V> {
    inner: Arc<Inner<V>>,
}

/// A type for tracking intermediate hashes at sparse merkle tree nodes in between batch
/// updates by transactions. It contains tuple (txn_id, hash_value, single_new_leaf), where
/// hash_value is the value after all the updates by transaction txn_id (txn_id-th batch)
/// and single_new_leaf is a bool that's true if the node subtree contains one new leaf.
/// (this is needed to recursively merge IntermediateHashes).
type IntermediateHashes = Vec<(usize, HashValue, bool)>;

impl<V> SparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
    /// Constructs a Sparse Merkle Tree with a root hash. This is often used when we restart and
    /// the scratch pad and the storage have identical state, so we use a single root hash to
    /// represent the entire state.
    pub fn new(root_hash: HashValue) -> Self {
        Self::new_impl(
            if root_hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                SubTree::new_unknown(root_hash)
            } else {
                SubTree::new_empty()
            },
            None,
        )
    }

    fn new_with_base(root: SubTree<V>, base: &Self) -> Self {
        Self::new_impl(root, Some(base.inner.clone()))
    }

    fn new_impl(root: SubTree<V>, base: Option<Arc<Inner<V>>>) -> Self {
        let inner = Inner {
            root: ArcSwap::from_pointee(root),
            base: ArcSwapOption::new(base),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    fn root_weak(&self) -> SubTree<V> {
        self.inner.root.load().weak()
    }

    /// Constructs a new Sparse Merkle Tree as if we are updating the existing tree. Since the tree
    /// is immutable, the existing tree will remain the same and may share part of the tree with
    /// the new one.
    pub fn update(
        &self,
        updates: Vec<(HashValue, &V)>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<Self, UpdateError> {
        updates
            .clone()
            .into_iter()
            .try_fold(self.clone(), |prev, (key, value)| {
                prev.update_one(key, value, proof_reader)
            })
    }

    /// Constructs a new Sparse Merkle Tree as if we are updating the existing tree multiple
    /// times with the `batch_update`. The function will return the root hash after each
    /// update and a Sparse Merkle Tree of the final state.
    ///
    /// The `serial_update` applies `batch_update' method many times, unlike a more optimized
    /// (and parallelizable) `batches_update' implementation below. It takes in a reference of
    /// value instead of an owned instance to be consistent with the `batches_update' interface.
    pub fn serial_update(
        &self,
        update_batch: Vec<Vec<(HashValue, &V)>>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<(Vec<HashValue>, Self), UpdateError> {
        let mut current_state_tree = self.clone();
        let mut result_hashes = Vec::with_capacity(update_batch.len());
        for updates in update_batch {
            current_state_tree = current_state_tree.batch_update(updates, proof_reader)?;
            result_hashes.push(current_state_tree.root_hash());
        }
        Ok((result_hashes, current_state_tree))
    }

    /// Constructs a new Sparse Merkle Tree, returns the SMT root hash after each update and the
    /// final SMT root. Since the tree is immutable, existing tree remains the same and may
    /// share parts with the new, returned tree. Unlike `serial_update', intermediate trees aren't
    /// constructed, but only root hashes are computed. `batches_update' takes value reference
    /// because the algorithm requires a copy per value at the end of tree traversals. Taking
    /// in a reference avoids double copy (by the caller and by the implementation).
    pub fn batches_update(
        &self,
        update_batch: Vec<Vec<(HashValue, &V)>>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<(Vec<HashValue>, Self), UpdateError> {
        let num_txns = update_batch.len();
        if num_txns == 0 {
            // No updates.
            return Ok((vec![], self.clone()));
        }

        // Construct (key, txn_id, value) update vector, where 0 <= txn_id < update_batch.len().
        // The entries are sorted and deduplicated, keeping last for each key per batch (txn).
        let updates: Vec<(HashValue, (usize, &V))> = update_batch
            .into_iter()
            .enumerate()
            .flat_map(|(txn_id, batch)| {
                batch
                    .into_iter()
                    .map(move |(hash, value)| ((hash, txn_id), value))
            })
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .map(|((hash, txn_id), value)| (hash, (txn_id, value))) // convert format.
            .collect();
        let root_weak = self.root_weak();
        let mut pre_hash = root_weak.hash();
        let (root, txn_hashes) = Self::batches_update_subtree(
            root_weak,
            /* subtree_depth = */ 0,
            &updates[..],
            proof_reader,
        )?;
        // Convert txn_hashes to the output format, i.e. a Vec<HashValue> holding a hash value
        // after each of the update_batch.len() many transactions.
        // - For transactions with no updates (i.e. root hash unchanged), txn_hashes don't have
        //  entries. So an updated hash value (txn_hashes.0) that remained the same after some
        //  transactions should be added to the result multiple times.
        // - If the first transactions didn't update, then pre-hash needs to be replicated.
        let mut txn_id = 0;
        let mut root_hashes = vec![];
        for txn_hash in &txn_hashes {
            while txn_id < txn_hash.0 {
                root_hashes.push(pre_hash);
                txn_id += 1;
            }
            pre_hash = txn_hash.1;
        }
        while txn_id < num_txns {
            root_hashes.push(pre_hash);
            txn_id += 1;
        }

        Ok((root_hashes, Self::new_with_base(root, self)))
    }

    /// Given an existing subtree node at a specific depth, recursively apply the updates.
    fn batches_update_subtree(
        subtree: SubTree<V>,
        subtree_depth: usize,
        updates: &[(HashValue, (usize, &V))],
        proof_reader: &impl ProofRead<V>,
    ) -> Result<(SubTree<V>, IntermediateHashes), UpdateError> {
        if updates.is_empty() {
            return Ok((subtree, vec![]));
        }

        if let SubTree::NonEmpty { root, .. } = &subtree {
            match root.get_node_if_in_mem() {
                Some(arc_node) => match arc_node.borrow() {
                    Node::Internal(internal_node) => {
                        let pivot = partition(updates, subtree_depth);
                        let left_weak = internal_node.left.weak();
                        let left_hash = left_weak.hash();
                        let right_weak = internal_node.right.weak();
                        let right_hash = right_weak.hash();
                        // TODO: parallelize calls up to a certain depth.
                        let (left_tree, left_hashes) = Self::batches_update_subtree(
                            left_weak,
                            subtree_depth + 1,
                            &updates[..pivot],
                            proof_reader,
                        )?;
                        let (right_tree, right_hashes) = Self::batches_update_subtree(
                            right_weak,
                            subtree_depth + 1,
                            &updates[pivot..],
                            proof_reader,
                        )?;

                        let merged_hashes = Self::merge_txn_hashes(
                            left_hash,
                            left_hashes,
                            right_hash,
                            right_hashes,
                        );
                        Ok((SubTree::new_internal(left_tree, right_tree), merged_hashes))
                    }
                    Node::Leaf(leaf_node) => Self::batch_create_subtree(
                        subtree.weak(), // 'root' is upgraded: OK to pass weak ptr.
                        /* target_key = */ leaf_node.key,
                        /* siblings = */ vec![],
                        subtree_depth,
                        updates,
                        proof_reader,
                    ),
                },
                // Subtree with hash only, need to use proofs.
                None => {
                    let (subtree, hashes, _) = Self::batch_create_subtree_by_proof(
                        updates,
                        proof_reader,
                        subtree.hash(),
                        subtree_depth,
                        *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    )?;
                    Ok((subtree, hashes))
                }
            }
        } else {
            // Subtree was empty.
            Self::batch_create_subtree(
                subtree.weak(), // 'root' is upgraded: OK to pass weak ptr.
                /* target_key = */ updates[0].0,
                /* siblings = */ vec![],
                subtree_depth,
                updates,
                proof_reader,
            )
        }
    }

    /// Generate a proof based on the first update and call 'batch_create_subtree' based
    /// on the proof's siblings and possibly a leaf. Additionally return the sibling hash of
    /// the subtree based on the proof (caller needs this information to merge hashes).
    fn batch_create_subtree_by_proof(
        updates: &[(HashValue, (usize, &V))],
        proof_reader: &impl ProofRead<V>,
        subtree_hash: HashValue,
        subtree_depth: usize,
        default_sibling_hash: HashValue,
    ) -> Result<(SubTree<V>, IntermediateHashes, HashValue), UpdateError> {
        if updates.is_empty() {
            return Ok((
                SubTree::new_unknown(subtree_hash),
                vec![],
                default_sibling_hash,
            ));
        }

        let update_key = updates[0].0;
        let proof = proof_reader
            .get_proof(update_key)
            .ok_or(UpdateError::MissingProof)?;
        let siblings: Vec<HashValue> = proof.siblings().iter().rev().copied().collect();

        let sibling_hash = if subtree_depth > 0 {
            *siblings
                .get(subtree_depth - 1)
                .unwrap_or(&SPARSE_MERKLE_PLACEHOLDER_HASH)
        } else {
            default_sibling_hash
        };

        let (subtree, hashes) = match proof.leaf() {
            Some(existing_leaf) => Self::batch_create_subtree(
                SubTree::new_leaf_with_value_hash(existing_leaf.key(), existing_leaf.value_hash()),
                /* target_key = */ existing_leaf.key(),
                siblings,
                subtree_depth,
                updates,
                proof_reader,
            )?,
            None => Self::batch_create_subtree(
                SubTree::new_empty(),
                /* target_key = */ update_key,
                siblings,
                subtree_depth,
                updates,
                proof_reader,
            )?,
        };

        Ok((subtree, hashes, sibling_hash))
    }

    /// Creates a new subtree. Important parameters are:
    /// - 'bottom_subtree' will be added at the bottom of the construction. It is either empty
    ///  or a leaf, containing either (a weak pointer to) a node from the previous version
    ///  that's being re-used, or (a strong pointer to) a leaf from a proof.
    /// - 'target_key' is the key of the bottom_subtree when bottom_subtree is a leaf, o.w. it
    ///  is the key of the first (leftmost) update.
    /// - 'siblings' are the siblings if bottom_subtree is a proof leaf, otherwise empty.
    fn batch_create_subtree(
        bottom_subtree: SubTree<V>,
        target_key: HashValue,
        siblings: Vec<HashValue>,
        subtree_depth: usize,
        updates: &[(HashValue, (usize, &V))],
        proof_reader: &impl ProofRead<V>,
    ) -> Result<(SubTree<V>, IntermediateHashes), UpdateError> {
        if updates.is_empty() {
            return Ok((bottom_subtree, vec![]));
        }
        if siblings.len() <= subtree_depth {
            if let Some(res) = Self::leaf_from_updates(target_key, updates) {
                return Ok(res);
            }
        }

        let pivot = partition(updates, subtree_depth);
        let child_is_right = target_key.bit(subtree_depth);
        let (child_updates, sibling_updates) =
            swap_if(&updates[..pivot], &updates[pivot..], child_is_right);

        let mut child_pre_hash = bottom_subtree.hash();
        let sibling_pre_hash = *siblings
            .get(subtree_depth)
            .unwrap_or(&SPARSE_MERKLE_PLACEHOLDER_HASH);

        // TODO: parallelize up to certain depth.
        let (sibling_tree, sibling_hashes) = if siblings.len() <= subtree_depth {
            // Implies sibling_pre_hash is empty.
            if sibling_updates.is_empty() {
                (SubTree::new_empty(), vec![])
            } else {
                Self::batch_create_subtree(
                    SubTree::new_empty(),
                    /* target_key = */ sibling_updates[0].0,
                    /* siblings = */ vec![],
                    subtree_depth + 1,
                    sibling_updates,
                    proof_reader,
                )?
            }
        } else {
            // Only have the sibling hash, need to use proofs.
            let (subtree, hashes, child_hash) = Self::batch_create_subtree_by_proof(
                sibling_updates,
                proof_reader,
                sibling_pre_hash,
                subtree_depth + 1,
                child_pre_hash,
            )?;
            child_pre_hash = child_hash;
            (subtree, hashes)
        };
        let (child_tree, child_hashes) = Self::batch_create_subtree(
            bottom_subtree,
            target_key,
            siblings,
            subtree_depth + 1,
            child_updates,
            proof_reader,
        )?;

        let (left_tree, right_tree) = swap_if(child_tree, sibling_tree, child_is_right);
        let (left_hashes, right_hashes) = swap_if(child_hashes, sibling_hashes, child_is_right);
        let (left_pre_hash, right_pre_hash) =
            swap_if(child_pre_hash, sibling_pre_hash, child_is_right);

        let merged_hashes =
            Self::merge_txn_hashes(left_pre_hash, left_hashes, right_pre_hash, right_hashes);
        Ok((SubTree::new_internal(left_tree, right_tree), merged_hashes))
    }

    /// Given a key and updates, checks if all updates are to this key. If so, generates
    /// a SubTree for a final leaf, and IntermediateHashes. Each intermediate update is by
    /// a different transaction as (key, txn_id) pairs are deduplicated.
    fn leaf_from_updates(
        leaf_key: HashValue,
        updates: &[(HashValue, (usize, &V))],
    ) -> Option<(SubTree<V>, IntermediateHashes)> {
        let first_update = updates.first().unwrap();
        let last_update = updates.last().unwrap();
        // Updates sorted by key: check that all keys are equal to leaf_key.
        if first_update.0 != leaf_key || last_update.0 != leaf_key {
            return None;
        };

        // Updates are to the same key and thus sorted by txn_id.
        let mut hashes: IntermediateHashes = updates
            .iter()
            .take(updates.len() - 1)
            .map(|&(_, (txn_id, value_ref))| {
                let value_hash = value_ref.hash();
                let leaf_hash = SparseMerkleLeafNode::new(leaf_key, value_hash).hash();
                (txn_id, leaf_hash, /* single_new_leaf = */ true)
            })
            .collect();
        let final_leaf =
            SubTree::new_leaf_with_value(leaf_key, last_update.1 .1.clone() /* value */);
        hashes.push((
            last_update.1 .0, /* txn_id */
            final_leaf.hash(),
            /* single_new_leaf = */ true,
        ));

        Some((final_leaf, hashes))
    }

    /// Given the hashes before updates, and IntermediateHashes for left and right Subtrees,
    /// compute IntermediateHashes for the parent node.
    fn merge_txn_hashes(
        left_pre_hash: HashValue,
        left_txn_hashes: IntermediateHashes,
        right_pre_hash: HashValue,
        right_txn_hashes: IntermediateHashes,
    ) -> IntermediateHashes {
        let (mut li, mut ri) = (0, 0);
        // Some lambda expressions for convenience.
        let next_txn_num = |i: usize, txn_hashes: &Vec<(usize, HashValue, bool)>| {
            if i < txn_hashes.len() {
                txn_hashes[i].0
            } else {
                usize::MAX
            }
        };
        let left_prev_txn_hash = |i: usize| {
            if i > 0 {
                left_txn_hashes[i - 1].1
            } else {
                left_pre_hash
            }
        };
        let right_prev_txn_hash = |i: usize| {
            if i > 0 {
                right_txn_hashes[i - 1].1
            } else {
                right_pre_hash
            }
        };

        let mut to_hash = vec![];
        while li < left_txn_hashes.len() || ri < right_txn_hashes.len() {
            let left_txn_num = next_txn_num(li, &left_txn_hashes);
            let right_txn_num = next_txn_num(ri, &right_txn_hashes);
            if left_txn_num <= right_txn_num {
                li += 1;
            }
            if right_txn_num <= left_txn_num {
                ri += 1;
            }

            // If one child was empty (based on previous hash) while the other child was
            // a single new leaf node, then the parent hash mustn't be combined. Instead,
            // it should be the single leaf hash (the leaf would have been added aerlier).
            let override_hash = if li > 0
                && left_txn_hashes[li - 1].2
                && ri == 0
                && right_pre_hash == *SPARSE_MERKLE_PLACEHOLDER_HASH
            {
                Some(left_prev_txn_hash(li))
            } else if ri > 0
                && right_txn_hashes[ri - 1].2
                && li == 0
                && left_pre_hash == *SPARSE_MERKLE_PLACEHOLDER_HASH
            {
                Some(right_prev_txn_hash(ri))
            } else {
                None
            };
            to_hash.push((
                cmp::min(left_txn_num, right_txn_num),
                left_prev_txn_hash(li),
                right_prev_txn_hash(ri),
                override_hash,
            ));
        }

        // TODO: parallelize w. par_iter.
        to_hash
            .iter()
            .map(|&(txn_num, left_hash, right_hash, override_hash)| {
                (
                    txn_num,
                    match override_hash {
                        Some(hash) => hash,
                        None => SparseMerkleInternalNode::new(left_hash, right_hash).hash(),
                    },
                    override_hash.is_some(),
                )
            })
            .collect()
    }

    fn update_one(
        &self,
        key: HashValue,
        new_value: &V,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<Self, UpdateError> {
        let mut current_subtree = self.root_weak();
        let mut bits = key.iter_bits();

        // Starting from root, traverse the tree according to key until we find a non-internal
        // node. Record all the bits and sibling nodes on the path.
        let mut bits_on_path = vec![];
        let mut siblings_on_path = vec![];
        loop {
            if let SubTree::NonEmpty { root, .. } = &current_subtree {
                if let Some(node) = root.get_node_if_in_mem() {
                    if let Node::Internal(internal_node) = node.borrow() {
                        let bit = bits.next().unwrap_or_else(|| {
                            // invariant of HashValueBitIterator
                            unreachable!(
                                "Tree is deeper than {} levels.",
                                HashValue::LENGTH_IN_BITS
                            )
                        });
                        bits_on_path.push(bit);
                        current_subtree = if bit {
                            siblings_on_path.push(internal_node.left.weak());
                            internal_node.right.weak()
                        } else {
                            siblings_on_path.push(internal_node.right.weak());
                            internal_node.left.weak()
                        };
                        continue;
                    }
                }
            }
            break;
        }

        // Now we are at the bottom of the tree and current_node can be either a leaf, unknown or
        // empty. We construct a new subtree like we are inserting the key here.
        let new_node = Self::construct_subtree_at_bottom(
            &current_subtree,
            key,
            new_value.clone(),
            bits,
            proof_reader,
        )?;

        // Use the new node and all previous siblings on the path to construct the final tree.
        let root = Self::construct_subtree(
            bits_on_path.into_iter().rev(),
            siblings_on_path.into_iter().rev(),
            new_node,
        );

        Ok(Self::new_with_base(root, self))
    }

    /// This function is called when we are trying to write (key, new_value) to the tree and have
    /// traversed the existing tree using some prefix of the key. We should have reached the bottom
    /// of the existing tree, so current_node cannot be an internal node. This function will
    /// construct a subtree using current_node, the new key-value pair and potentially the
    /// key-value pair in the proof.
    fn construct_subtree_at_bottom(
        current_subtree: &SubTree<V>,
        key: HashValue,
        new_value: V,
        remaining_bits: HashValueBitIterator,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<SubTree<V>, UpdateError> {
        match current_subtree {
            SubTree::Empty => {
                // When we reach an empty node, we just place the leaf node at the same position to
                // replace the empty node.
                Ok(SubTree::new_leaf_with_value(key, new_value))
            }
            SubTree::NonEmpty { root, .. } => {
                match root.get_node_if_in_mem() {
                    Some(node) => match node.borrow() {
                        Node::Internal(_) => {
                            unreachable!("Reached an internal node at the bottom of the tree.");
                        }
                        Node::Leaf(leaf_node) => Ok(Self::construct_subtree_with_new_leaf(
                            key,
                            new_value,
                            current_subtree.weak(),
                            leaf_node.key,
                            HashValue::LENGTH_IN_BITS
                                .checked_sub(remaining_bits.len())
                                .expect("shouldn't overflow"),
                        )),
                    },
                    None => {
                        // When the search reaches an unknown subtree, we need proof to give us more
                        // information about this part of the tree.
                        let proof = proof_reader
                            .get_proof(key)
                            .ok_or(UpdateError::MissingProof)?;

                        // Here the in-memory tree is identical to the tree in storage (we have only the
                        // root hash of this subtree in memory). So we need to take into account the leaf
                        // in the proof.
                        let new_subtree = match proof.leaf() {
                            Some(existing_leaf) => Self::construct_subtree_with_new_leaf(
                                key,
                                new_value,
                                SubTree::new_leaf_with_value_hash(
                                    existing_leaf.key(),
                                    existing_leaf.value_hash(),
                                ),
                                existing_leaf.key(),
                                proof.siblings().len(),
                            ),
                            None => SubTree::new_leaf_with_value(key, new_value),
                        };

                        let num_remaining_bits = remaining_bits.len();
                        let proof_length = proof.siblings().len();
                        Ok(Self::construct_subtree(
                            remaining_bits.rev().skip(
                                HashValue::LENGTH_IN_BITS
                                    .checked_sub(proof_length)
                                    .expect("shouldn't overflow"),
                            ),
                            proof
                                .siblings()
                                .iter()
                                .take(
                                    num_remaining_bits
                                        .checked_add(proof_length)
                                        .expect("shouldn't overflow")
                                        .checked_sub(HashValue::LENGTH_IN_BITS)
                                        .expect("shouldn't overflow"),
                                )
                                .map(|sibling_hash| {
                                    if *sibling_hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                                        SubTree::new_unknown(*sibling_hash)
                                    } else {
                                        SubTree::new_empty()
                                    }
                                }),
                            new_subtree,
                        ))
                    }
                }
            }
        }
    }

    /// Given key, new value, existing leaf and the distance from root to the existing leaf,
    /// constructs a new subtree that has either the new leaf or both nodes, depending on whether
    /// the key equals the existing leaf's key.
    ///
    /// 1. If the key equals the existing leaf's key, we simply need to update the leaf to the new
    ///    value and return it. For example, in the following case this function will return
    ///    `new_leaf`.
    ///  ``` text
    ///       o                    o
    ///      / \                  / \
    ///     o   o       =>       o   o
    ///    / \                  / \
    ///   o   existing_leaf    o   new_leaf
    ///  ```
    ///
    /// 2. Otherwise, we need to construct an "extension" for the common prefix, and at the end of
    ///    the extension a subtree for both keys. For example, in the following case we assume the
    ///    existing leaf's key starts with 010010 and key starts with 010011, and this function
    ///    will return `x`.
    /// ```text
    ///        o                              o             common_prefix_len = 5
    ///       / \                            / \            distance_from_root_to_existing_leaf = 2
    ///      o   o                          o   o           extension_len = common_prefix_len - distance_from_root_to_existing_leaf = 3
    ///     / \                            / \
    ///    o   existing_leaf    =>        o   x                 _
    ///                                      / \                ^
    ///                                     o   placeholder     |
    ///                                    / \                  |
    ///                                   o   placeholder   extension
    ///                                  / \                    |
    ///                       placeholder   o                   -
    ///                                    / \
    ///                       existing_leaf   new_leaf
    /// ```
    fn construct_subtree_with_new_leaf(
        key: HashValue,
        new_value: V,
        existing_leaf: SubTree<V>,
        existing_leaf_key: HashValue,
        distance_from_root_to_existing_leaf: usize,
    ) -> SubTree<V> {
        let new_leaf = SubTree::new_leaf_with_value(key, new_value);
        if key == existing_leaf_key {
            // This implies that `key` already existed and the proof is an inclusion proof.
            return new_leaf;
        }

        // This implies that `key` did not exist and was just created. The proof is a non-inclusion
        // proof. See above example for how extension_len is computed.
        let common_prefix_len = key.common_prefix_bits_len(existing_leaf_key);
        assert!(
            common_prefix_len >= distance_from_root_to_existing_leaf,
            "common_prefix_len: {}, distance_from_root_to_existing_leaf: {}",
            common_prefix_len,
            distance_from_root_to_existing_leaf,
        );
        let extension_len = common_prefix_len - distance_from_root_to_existing_leaf;
        Self::construct_subtree(
            key.iter_bits()
                .rev()
                .skip(HashValue::LENGTH_IN_BITS - common_prefix_len - 1)
                .take(extension_len + 1),
            std::iter::once(existing_leaf)
                .chain(std::iter::repeat(SubTree::new_empty()).take(extension_len)),
            new_leaf,
        )
    }

    /// Constructs a subtree with a list of siblings and a leaf. For example, if `bits` are
    /// [false, false, true] and `siblings` are [a, b, c], the resulting subtree will look like:
    /// ```text
    ///          x
    ///         / \
    ///        c   o
    ///           / \
    ///          o   b
    ///         / \
    ///     leaf   a
    /// ```
    /// and this function will return `x`. Both `bits` and `siblings` start from the bottom.
    fn construct_subtree(
        bits: impl Iterator<Item = bool>,
        siblings: impl Iterator<Item = SubTree<V>>,
        leaf: SubTree<V>,
    ) -> SubTree<V> {
        itertools::zip_eq(bits, siblings).fold(leaf, |previous_node, (bit, sibling)| {
            if bit {
                SubTree::new_internal(sibling, previous_node)
            } else {
                SubTree::new_internal(previous_node, sibling)
            }
        })
    }

    /// Queries a `key` in this `SparseMerkleTree`.
    pub fn get(&self, key: HashValue) -> AccountStatus<V> {
        let mut cur = self.root_weak();
        let mut bits = key.iter_bits();

        loop {
            if let Some(node) = cur.get_node_if_in_mem() {
                if let Node::Internal(internal_node) = node.borrow() {
                    match bits.next() {
                        Some(bit) => {
                            cur = if bit {
                                internal_node.right.weak()
                            } else {
                                internal_node.left.weak()
                            };
                            continue;
                        }
                        None => panic!("Tree is deeper than {} levels.", HashValue::LENGTH_IN_BITS),
                    }
                }
            }
            break;
        }

        let ret = match cur {
            SubTree::Empty => AccountStatus::DoesNotExist,
            SubTree::NonEmpty { root, .. } => match root.get_node_if_in_mem() {
                None => AccountStatus::Unknown,
                Some(node) => match node.borrow() {
                    Node::Internal(_) => {
                        unreachable!("There is an internal node at the bottom of the tree.")
                    }
                    Node::Leaf(leaf_node) => {
                        if leaf_node.key == key {
                            match &leaf_node.value {
                                LeafValue::Value(value) => {
                                    AccountStatus::ExistsInScratchPad(value.clone())
                                }
                                LeafValue::ValueHash(_) => AccountStatus::ExistsInDB,
                            }
                        } else {
                            AccountStatus::DoesNotExist
                        }
                    }
                },
            },
        };
        ret
    }

    /// The batch version of `serial_update`. The function will return a Sparse Merkle Tree of the final state.
    /// This method is optmized to avoid repetitive common path updating and parallelize the processing as much
    /// as possible.
    pub fn batch_update(
        &self,
        updates: Vec<(HashValue, &V)>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<Self, UpdateError> {
        // Flatten, dedup and sort the updates with a btree map since the updates between different
        // versions may overlap on the same address in which case the latter always overwrites.
        let kvs = updates
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect::<Vec<_>>();

        let current_root = self.root_weak();
        if kvs.is_empty() {
            Ok(self.clone())
        } else {
            let root = Self::batch_update_subtree(
                current_root,
                &kvs[..],
                /* depth = */ 0,
                proof_reader,
                /* is_generated_from_proofs = */ false,
            )?;
            Ok(Self::new_with_base(root, self))
        }
    }

    /// Given a current subtree node with its depth and a list of updates with key and value, update the
    /// subtree according to the new k-v pairs.
    fn batch_update_subtree(
        mut current_subtree: SubTree<V>,
        kvs: &[(HashValue, &V)],
        mut depth: usize,
        proof_reader: &impl ProofRead<V>,
        is_generated_from_proofs: bool,
    ) -> Result<SubTree<V>, UpdateError> {
        let n = kvs.len();

        // Starting from root, traverse the tree according to key until we find a non-internal
        // node. Record all the bits and sibling nodes on the path.
        let mut bits_on_path = vec![];
        let mut siblings_on_path = vec![];
        let new_node = loop {
            assert!(
                depth < HashValue::LENGTH_IN_BITS,
                "depth {} cannot be deeper than {}",
                depth,
                HashValue::LENGTH_IN_BITS - 1,
            );
            match &current_subtree {
                SubTree::NonEmpty { root, .. } => {
                    if let Some(node) = root.get_node_if_in_mem() {
                        match node.borrow() {
                            Node::Internal(internal_node) => {
                                let idx = partition(kvs, depth);
                                let (left, right) = if is_generated_from_proofs {
                                    (internal_node.left.clone(), internal_node.right.clone())
                                } else {
                                    (internal_node.left.weak(), internal_node.right.weak())
                                };
                                depth += 1;
                                if idx == 0 {
                                    siblings_on_path.push(left);
                                    current_subtree = right;
                                    bits_on_path.push(true);
                                } else if idx == n {
                                    siblings_on_path.push(right);
                                    current_subtree = left;
                                    bits_on_path.push(false);
                                } else {
                                    let (left_child, right_child) = rayon::join(
                                        || {
                                            Self::batch_update_subtree(
                                                left,
                                                &kvs[..idx],
                                                depth,
                                                proof_reader,
                                                is_generated_from_proofs,
                                            )
                                        },
                                        || {
                                            Self::batch_update_subtree(
                                                right,
                                                &kvs[idx..],
                                                depth,
                                                proof_reader,
                                                is_generated_from_proofs,
                                            )
                                        },
                                    );
                                    break SubTree::new_internal(left_child?, right_child?);
                                }
                            }
                            Node::Leaf(leaf_node) => {
                                break Self::batch_construct_subtree_with_existing_leaf(
                                    kvs,
                                    if is_generated_from_proofs {
                                        current_subtree.clone()
                                    } else {
                                        current_subtree.weak()
                                    },
                                    leaf_node.key,
                                    depth,
                                );
                            }
                        }
                    } else {
                        assert_eq!(is_generated_from_proofs, false);
                        // When the search reaches an unknown subtree, we need proof to give us more
                        // information about this part of the tree.
                        let mut key_proof_pairs: Vec<(HashValue, &SparseMerkleProof<V>)> = vec![];
                        for kv in kvs.iter() {
                            if let Some((last_key, last_proof)) = key_proof_pairs.last() {
                                let num_siblings = last_proof.siblings().len();
                                if last_proof
                                    .leaf()
                                    .map_or(*last_key, |leaf| leaf.key())
                                    .common_prefix_bits_len(kv.0)
                                    >= num_siblings
                                {
                                    // This proof is also applicable to the current key
                                    continue;
                                }
                            }
                            let proof = proof_reader
                                .get_proof(kv.0)
                                .ok_or(UpdateError::MissingProof)?;
                            key_proof_pairs.push((kv.0, proof));
                        }

                        let subtree_from_proofs = Self::batch_create_subtree_from_key_proof_pairs(
                            key_proof_pairs.as_slice(),
                            depth,
                        );

                        break Self::batch_update_subtree(
                            subtree_from_proofs,
                            kvs,
                            depth,
                            proof_reader,
                            /* is_generated_from_proofs = */ true,
                        )?;
                    }
                }
                SubTree::Empty => break Self::batch_construct_subtree_from_empty(kvs, depth),
            }
        };

        // Use the new node and all previous siblings on the path to construct the final tree.
        Ok(Self::construct_subtree(
            bits_on_path.into_iter().rev(),
            siblings_on_path.into_iter().rev(),
            new_node,
        ))
    }

    /// This function is called when we are trying to write Vec<(key, new_value)> to the tree and have
    /// traversed the existing tree using a common prefix of the keys. We should have reached the bottom
    /// of the existing tree, so current_node cannot be an internal node. This function will
    /// construct a subtree using current_node, all the new key-value pairs that share a common path above
    /// current_node and potentially key-value pairs in the necessary proofs.
    /// This function create a subtree at an empty node at `depth` with a list of k-v pairs.
    fn batch_construct_subtree_from_empty(kvs: &[(HashValue, &V)], mut depth: usize) -> SubTree<V> {
        let n = kvs.len();
        assert!(
            n != 0,
            "There must be at least 1 KV-pair to create a subtree"
        );
        if n == 1 {
            SubTree::new_leaf_with_value(kvs[0].0, kvs[0].1.clone())
        } else {
            let mut bits_on_path = vec![];
            loop {
                let idx = partition(kvs, depth);
                depth += 1;
                if idx == 0 {
                    bits_on_path.push(true);
                } else if idx == n {
                    bits_on_path.push(false);
                } else {
                    let (left_child, right_child) = rayon::join(
                        || Self::batch_construct_subtree_from_empty(&kvs[..idx], depth),
                        || Self::batch_construct_subtree_from_empty(&kvs[idx..], depth),
                    );
                    let fork_node = SubTree::new_internal(left_child, right_child);
                    let siblings_num = bits_on_path.len();
                    return Self::construct_subtree(
                        bits_on_path.into_iter().rev(),
                        std::iter::repeat(SubTree::new_empty()).take(siblings_num),
                        fork_node,
                    );
                }
            }
        }
    }

    /// This function reconstruct a partial tree in memory from the proofs provided, which can be merged
    /// later with k-v pairs matching the proofs. Usually this happens when trying to insert multple
    /// k-v pairs at a unknown node. Instead of sequentially update the subtree node, in a batching way,
    /// all the proofs can form a comprehensive view of the current subtree in storage.
    fn batch_create_subtree_from_key_proof_pairs(
        key_proof_pairs: &[(HashValue, &SparseMerkleProof<V>)],
        mut depth: usize,
    ) -> SubTree<V> {
        let n = key_proof_pairs.len();
        assert!(n != 0, "There must be at least 1 proof to create a subtree");
        if n == 1 {
            let (key, proof) = key_proof_pairs[0];
            let end_node = match proof.leaf() {
                Some(existing_leaf) => SubTree::new_leaf_with_value_hash(
                    existing_leaf.key(),
                    existing_leaf.value_hash(),
                ),
                None => SubTree::new_empty(),
            };
            let proof_length = proof.siblings().len();
            assert!(
                proof_length >= depth,
                "proof length cannot be shorter than depth"
            );
            Self::construct_subtree(
                key.iter_bits().skip(depth).rev().skip(
                    HashValue::LENGTH_IN_BITS
                        .checked_sub(proof_length)
                        .expect("shouldn't overflow"),
                ),
                proof
                    .siblings()
                    .iter()
                    .take(proof_length.checked_sub(depth).expect("shouldn't overflow"))
                    .map(|sibling_hash| {
                        if *sibling_hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                            SubTree::new_unknown(*sibling_hash)
                        } else {
                            SubTree::new_empty()
                        }
                    }),
                end_node,
            )
        } else {
            let mut bits_on_path = vec![];
            let mut siblings_on_path = vec![];
            loop {
                let idx = partition(key_proof_pairs, depth);
                let siblings = key_proof_pairs[0].1.siblings();
                if idx == 0 || idx == n {
                    let sibling_hash = siblings[siblings.len() - 1 - depth];
                    siblings_on_path.push(if sibling_hash == *SPARSE_MERKLE_PLACEHOLDER_HASH {
                        SubTree::new_empty()
                    } else {
                        SubTree::new_unknown(sibling_hash)
                    });
                    bits_on_path.push(idx == 0);
                } else {
                    let (left_child, right_child) = rayon::join(
                        || {
                            Self::batch_create_subtree_from_key_proof_pairs(
                                &key_proof_pairs[..idx],
                                depth + 1,
                            )
                        },
                        || {
                            Self::batch_create_subtree_from_key_proof_pairs(
                                &key_proof_pairs[idx..],
                                depth + 1,
                            )
                        },
                    );
                    let fork_node = SubTree::new_internal(left_child, right_child);
                    return Self::construct_subtree(
                        bits_on_path.into_iter().rev(),
                        siblings_on_path.into_iter().rev(),
                        fork_node,
                    );
                }
                depth += 1;
            }
        }
    }

    /// This function merge a leaf node with a list of k-v pairs.
    fn batch_construct_subtree_with_existing_leaf(
        kvs: &[(HashValue, &V)],
        existing_leaf: SubTree<V>,
        existing_leaf_key: HashValue,
        mut depth: usize,
    ) -> SubTree<V> {
        let n = kvs.len();
        assert!(
            n != 0,
            "There must be at least 1 KV-pair to create a subtree"
        );
        if n == 1 {
            Self::construct_subtree_with_new_leaf(
                kvs[0].0,
                kvs[0].1.clone(),
                existing_leaf,
                existing_leaf_key,
                depth,
            )
        } else {
            let mut bits_on_path = vec![];
            loop {
                let idx = partition(kvs, depth);
                if idx == 0 {
                    if existing_leaf_key.bit(depth) {
                        bits_on_path.push(true);
                    } else {
                        let siblings_num = bits_on_path.len();
                        return Self::construct_subtree(
                            bits_on_path.into_iter().rev(),
                            std::iter::repeat(SubTree::new_empty()).take(siblings_num),
                            SubTree::new_internal(
                                existing_leaf,
                                Self::batch_construct_subtree_from_empty(kvs, depth + 1),
                            ),
                        );
                    }
                } else if idx == n {
                    if !existing_leaf_key.bit(depth) {
                        bits_on_path.push(false);
                    } else {
                        let siblings_num = bits_on_path.len();
                        return Self::construct_subtree(
                            bits_on_path.into_iter().rev(),
                            std::iter::repeat(SubTree::new_empty()).take(siblings_num),
                            SubTree::new_internal(
                                Self::batch_construct_subtree_from_empty(kvs, depth + 1),
                                existing_leaf,
                            ),
                        );
                    }
                } else {
                    let (left_child, right_child) = if existing_leaf_key.bit(depth) {
                        rayon::join(
                            || Self::batch_construct_subtree_from_empty(&kvs[..idx], depth + 1),
                            || {
                                Self::batch_construct_subtree_with_existing_leaf(
                                    &kvs[idx..],
                                    existing_leaf,
                                    existing_leaf_key,
                                    depth + 1,
                                )
                            },
                        )
                    } else {
                        rayon::join(
                            || {
                                Self::batch_construct_subtree_with_existing_leaf(
                                    &kvs[..idx],
                                    existing_leaf,
                                    existing_leaf_key,
                                    depth + 1,
                                )
                            },
                            || Self::batch_construct_subtree_from_empty(&kvs[idx..], depth + 1),
                        )
                    };
                    let fork_node = SubTree::new_internal(left_child, right_child);
                    let siblings_num = bits_on_path.len();
                    return Self::construct_subtree(
                        bits_on_path.into_iter().rev(),
                        std::iter::repeat(SubTree::new_empty()).take(siblings_num),
                        fork_node,
                    );
                }
                depth += 1;
            }
        }
    }

    /// Returns the root hash of this tree.
    pub fn root_hash(&self) -> HashValue {
        self.inner.root.load().hash()
    }

    /// Mark that all the nodes created by this tree and its ancestors are persisted in the DB.
    pub fn prune(&self) {
        self.inner.prune()
    }
}

impl<V> Default for SparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
    fn default() -> Self {
        SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH)
    }
}

/// A type that implements `ProofRead` can provide proof for keys in persistent storage.
pub trait ProofRead<V>: Sync {
    /// Gets verified proof for this key in persistent storage.
    fn get_proof(&self, key: HashValue) -> Option<&SparseMerkleProof<V>>;
}

/// All errors `update` can possibly return.
#[derive(Debug, Eq, PartialEq)]
pub enum UpdateError {
    /// The update intends to insert a key that does not exist in the tree, so the operation needs
    /// proof to get more information about the tree, but no proof is provided.
    MissingProof,
}
