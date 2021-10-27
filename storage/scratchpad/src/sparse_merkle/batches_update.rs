// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::SparseMerkleTree;
use crate::{
    sparse_merkle::{
        node::{NodeInner, SubTree},
        utils::{partition, swap_if},
        IntermediateHashes, UpdateError,
    },
    ProofRead,
};
use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_types::proof::{SparseMerkleInternalNode, SparseMerkleLeafNode};
use std::{borrow::Borrow, cmp, collections::BTreeMap};

impl<V> SparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
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
            self.inner.generation + 1,
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

        Ok((root_hashes, self.spawn(root)))
    }

    /// Given an existing subtree node at a specific depth, recursively apply the updates.
    fn batches_update_subtree(
        subtree: SubTree<V>,
        subtree_depth: usize,
        updates: &[(HashValue, (usize, &V))],
        proof_reader: &impl ProofRead<V>,
        generation: u64,
    ) -> Result<(SubTree<V>, IntermediateHashes), UpdateError> {
        if updates.is_empty() {
            return Ok((subtree, vec![]));
        }

        if let SubTree::NonEmpty { root, .. } = &subtree {
            match root.get_if_in_mem() {
                Some(arc_node) => match arc_node.inner().borrow() {
                    NodeInner::Internal(internal_node) => {
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
                            generation,
                        )?;
                        let (right_tree, right_hashes) = Self::batches_update_subtree(
                            right_weak,
                            subtree_depth + 1,
                            &updates[pivot..],
                            proof_reader,
                            generation,
                        )?;

                        let merged_hashes = Self::merge_txn_hashes(
                            left_hash,
                            left_hashes,
                            right_hash,
                            right_hashes,
                        );
                        Ok((
                            SubTree::new_internal(left_tree, right_tree, generation),
                            merged_hashes,
                        ))
                    }
                    NodeInner::Leaf(leaf_node) => Self::batch_create_subtree(
                        subtree.weak(), // 'root' is upgraded: OK to pass weak ptr.
                        /* target_key = */ leaf_node.key,
                        /* siblings = */ vec![],
                        subtree_depth,
                        updates,
                        proof_reader,
                        generation,
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
                        generation,
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
                generation,
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
        generation: u64,
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
                SubTree::new_leaf_with_value_hash(
                    existing_leaf.key(),
                    existing_leaf.value_hash(),
                    generation,
                ),
                /* target_key = */ existing_leaf.key(),
                siblings,
                subtree_depth,
                updates,
                proof_reader,
                generation,
            )?,
            None => Self::batch_create_subtree(
                SubTree::new_empty(),
                /* target_key = */ update_key,
                siblings,
                subtree_depth,
                updates,
                proof_reader,
                generation,
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
        generation: u64,
    ) -> Result<(SubTree<V>, IntermediateHashes), UpdateError> {
        if updates.is_empty() {
            return Ok((bottom_subtree, vec![]));
        }
        if siblings.len() <= subtree_depth {
            if let Some(res) = Self::leaf_from_updates(target_key, updates, generation) {
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
                    generation,
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
                generation,
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
            generation,
        )?;

        let (left_tree, right_tree) = swap_if(child_tree, sibling_tree, child_is_right);
        let (left_hashes, right_hashes) = swap_if(child_hashes, sibling_hashes, child_is_right);
        let (left_pre_hash, right_pre_hash) =
            swap_if(child_pre_hash, sibling_pre_hash, child_is_right);

        let merged_hashes =
            Self::merge_txn_hashes(left_pre_hash, left_hashes, right_pre_hash, right_hashes);
        Ok((
            SubTree::new_internal(left_tree, right_tree, generation),
            merged_hashes,
        ))
    }

    /// Given a key and updates, checks if all updates are to this key. If so, generates
    /// a SubTree for a final leaf, and IntermediateHashes. Each intermediate update is by
    /// a different transaction as (key, txn_id) pairs are deduplicated.
    fn leaf_from_updates(
        leaf_key: HashValue,
        updates: &[(HashValue, (usize, &V))],
        generation: u64,
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
        let final_leaf = SubTree::new_leaf_with_value(
            leaf_key,
            last_update.1 .1.clone(), /* value */
            generation,
        );
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
}
