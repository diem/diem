// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines state store APIs that are related account state Merkle tree.

// FIXME(aldenhu): remove
#![allow(dead_code)]

use crate::{
    account_state_blob::AccountStateBlobSchema,
    bsmt_update_log::{BsmtKey, BsmtUpdateLogSchema},
    state_tree::StateTreeSchema,
    ChangeSet, DiemDbError, DIEM_STORAGE_OTHER_TIMERS_SECONDS,
};
use anyhow::Result;
use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_types::{
    account_address::HashAccountAddress,
    account_state_blob::{AccountStateBlob, AccountStatesChunkWithProof},
    proof::{
        definition::LeafCount, SparseMerkleLeafNode, SparseMerkleProof, SparseMerkleRangeProof,
    },
    transaction::{StateTreeNode, StateTreeNodeType, Version, PRE_GENESIS_VERSION},
    vec_bool::VecBool,
};
use itertools::zip_eq;
use move_core_types::account_address::AccountAddress;
use rayon::prelude::*;
use schemadb::DB;
use std::{collections::HashMap, sync::Arc};
use storage_interface::StateSnapshotReceiver;

/*
#[cfg(test)]
mod state_store_test;
 */

pub(crate) mod pruner;

#[derive(Debug)]
pub(crate) struct StateStore {
    db: Arc<DB>,
}

impl StateStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub fn get_account_state_with_partial_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
        starting_depth: usize,
    ) -> Result<(
        Option<AccountStateBlob>,
        SparseMerkleProof<AccountStateBlob>,
    )> {
        self.get_account_state_with_partial_proof_by_version_impl(address, version, starting_depth)
            .map(|(blob, proof)| {
                DIEM_STORAGE_OTHER_TIMERS_SECONDS
                    .with_label_values(&["get_partial_proof_starting_depth"])
                    .observe(starting_depth as f64);
                DIEM_STORAGE_OTHER_TIMERS_SECONDS
                    .with_label_values(&["get_partial_proof_bottom_depth"])
                    .observe(proof.bottom_depth() as f64);
                (blob, proof)
            })
    }

    pub fn get_account_state_with_partial_proof_by_version_impl(
        &self,
        address: AccountAddress,
        version: Version,
        starting_depth: usize,
    ) -> Result<(
        Option<AccountStateBlob>,
        SparseMerkleProof<AccountStateBlob>,
    )> {
        assert!(starting_depth <= 256);
        let target_key = address.hash();

        let mut pos = VecBool::new();
        let mut bit_iter = target_key.iter_bits();
        for _ in 0..starting_depth {
            let bit = bit_iter.next().unwrap();
            pos.push(bit)
        }

        let root = self.get_node_before(version, &pos)?;
        let (mut left_empty, mut right_empty) = match &root.node_type {
            // FIXME(aldenhu): for separate blob and proof fetches, store value hash in leaf, to avoid a blob read
            StateTreeNodeType::Leaf { key: leaf_key } => {
                let blob = self.leaf(*leaf_key, version)?;
                let value_hash = blob.hash();
                return Ok((
                    if *leaf_key == target_key {
                        Some(blob)
                    } else {
                        None
                    },
                    SparseMerkleProof::new_partial(
                        Some(SparseMerkleLeafNode::new(*leaf_key, value_hash)),
                        Vec::new(),
                        starting_depth,
                    ),
                ));
            }
            StateTreeNodeType::Internal {
                left_empty,
                right_empty,
            } => (*left_empty, *right_empty),
        };

        let mut siblings = Vec::new();
        for bit in bit_iter {
            let (me_empty, sibling_empty) = swap_if(left_empty, right_empty, bit);
            siblings.push(if sibling_empty {
                *SPARSE_MERKLE_PLACEHOLDER_HASH
            } else {
                pos.push(!bit);
                let sibling = self.get_node_before(version, &pos)?;
                pos.pop();
                sibling.hash
            });

            if me_empty {
                return Ok((
                    None,
                    SparseMerkleProof::new_partial(
                        None,
                        {
                            siblings.reverse();
                            siblings
                        },
                        starting_depth,
                    ),
                ));
            }

            pos.push(bit);
            let node = self.get_node_before(version, &pos)?;
            match &node.node_type {
                StateTreeNodeType::Leaf { key } => {
                    let blob = self.leaf(*key, version)?;
                    let value_hash = blob.hash();
                    return Ok((
                        if *key == target_key { Some(blob) } else { None },
                        SparseMerkleProof::new_partial(
                            Some(SparseMerkleLeafNode::new(*key, value_hash)),
                            {
                                siblings.reverse();
                                siblings
                            },
                            starting_depth,
                        ),
                    ));
                }
                StateTreeNodeType::Internal {
                    left_empty: l,
                    right_empty: r,
                } => {
                    left_empty = *l;
                    right_empty = *r;
                }
            }
        }

        unreachable!("Should've seen a leaf and returned already.")
    }

    fn leaf(&self, key: HashValue, max_version: Version) -> Result<AccountStateBlob> {
        let mut iter = self.db.iter::<AccountStateBlobSchema>(Default::default())?;
        iter.seek(&(key, max_version))?;

        Ok(iter
            .next()
            .transpose()?
            .map(|((_key, _ver), blob)| {
                // FIXME(aldenhu): proper error
                assert_eq!(_key, key);
                blob
            })
            .expect("Not found."))
    }

    /// Gets the proof that proves a range of accounts.
    pub fn get_account_state_range_proof(
        &self,
        _rightmost_key: HashValue,
        _version: Version,
    ) -> Result<SparseMerkleRangeProof> {
        unreachable!()
    }

    /// Put the results generated by `account_state_sets` to `batch` and return the result root
    /// hashes for each write set.
    pub fn put_account_state_sets(
        &self,
        account_state_sets: Vec<HashMap<AccountAddress, AccountStateBlob>>,
        nodes_vecs: Vec<&[(VecBool, StateTreeNode)]>,
        first_version: Version,
    ) -> Result<ChangeSet> {
        let _timer = DIEM_STORAGE_OTHER_TIMERS_SECONDS
            .with_label_values(&["state_store_put_account_state_sets"])
            .start_timer();

        let cs_vec: Vec<_> = zip_eq(account_state_sets, nodes_vecs)
            .enumerate()
            .collect::<Vec<_>>()
            .into_par_iter()
            .with_min_len(100)
            .map(|(idx, (account_state_set, nodes_vec))| {
                let cs = ChangeSet::new();
                let version = first_version + idx as LeafCount;

                let cs = account_state_set
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .with_min_len(100)
                    .map(|(addr, blob)| {
                        let mut cs = ChangeSet::new();
                        // FIXME(aldenhu): addr.hash() doesn't need to be in value?
                        // FIXME(aldenhu): pass hashes along?
                        // FIXME(aldenhu): addr.hash() as input directly instead of addr?
                        let key = addr.hash();
                        cs.batch
                            .put::<AccountStateBlobSchema>(&(key, version), &blob)?;
                        cs.batch
                            .put::<BsmtUpdateLogSchema>(&(version, BsmtKey::Value(key)), &())?;
                        Ok(cs)
                    })
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .fold(cs, ChangeSet::fold);

                let cs = nodes_vec
                    .into_par_iter()
                    .with_min_len(100)
                    .map(|(node_addr, node)| {
                        let mut cs = ChangeSet::new();
                        cs.batch
                            .put::<StateTreeSchema>(&(node_addr.clone(), version), node)?;
                        cs.batch.put::<BsmtUpdateLogSchema>(
                            &(version, BsmtKey::Node(node_addr.clone())),
                            &(),
                        )?;
                        Ok(cs)
                    })
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .fold(cs, ChangeSet::fold);
                Ok(cs)
            })
            .collect::<Result<_>>()?;

        Ok(cs_vec.into_iter().fold(ChangeSet::new(), ChangeSet::fold))
    }

    pub fn get_root_hash(&self, version: Version) -> Result<HashValue> {
        self.get_root_node(version).map(|node| node.hash)
    }

    pub fn get_root_hash_option(&self, version: Version) -> Result<Option<HashValue>> {
        Ok(self.get_root_node_option(version)?.map(|node| node.hash))
    }

    pub fn get_root_node(&self, version: Version) -> Result<StateTreeNode> {
        self.get_root_node_option(version)?.ok_or_else(|| {
            DiemDbError::NotFound(format!("Root node for version {}", version)).into()
        })
    }

    pub fn get_root_node_option(&self, version: Version) -> Result<Option<StateTreeNode>> {
        self.db.get::<StateTreeSchema>(&(VecBool::new(), version))
    }

    pub fn get_node_before(&self, max_version: Version, pos: &VecBool) -> Result<StateTreeNode> {
        self.get_node_option(max_version, pos)?.ok_or_else(|| {
            // FIXME(aldenhu): impl Display for VecBool
            DiemDbError::NotFound(format!(
                "StateNode at pos {:?}, max_version {}",
                pos, max_version
            ))
            .into()
        })
    }

    pub fn get_node_option(
        &self,
        max_version: Version,
        pos: &VecBool,
    ) -> Result<Option<StateTreeNode>> {
        let mut iter = self.db.iter::<StateTreeSchema>(Default::default())?;
        iter.seek(&(pos.clone(), max_version))?;

        Ok(iter.next().transpose()?.and_then(
            |((p, _ver), node)| {
                if p == *pos {
                    Some(node)
                } else {
                    None
                }
            },
        ))
    }

    pub fn get_latest_checkpoint(
        &self,
        max_num_transactions: LeafCount,
    ) -> Result<(LeafCount, HashValue)> {
        if max_num_transactions == 0 {
            let root_hash = self
                .get_root_hash_option(PRE_GENESIS_VERSION)?
                .unwrap_or(*SPARSE_MERKLE_PLACEHOLDER_HASH);
            Ok((0, root_hash))
        } else {
            let mut iter = self.db.iter::<StateTreeSchema>(Default::default())?;
            iter.seek(&(VecBool::new(), max_num_transactions - 1))?;
            let ((_, version), node) = iter.next().transpose()?.ok_or_else(|| {
                DiemDbError::NotFound(format!(
                    "State checkpoint before version {}",
                    max_num_transactions
                ))
            })?;
            Ok((version + 1, node.hash))
        }
    }

    pub fn get_account_count(&self, _version: Version) -> Result<Option<usize>> {
        Ok(None)
    }

    pub fn get_account_chunk_with_proof(
        self: &Arc<Self>,
        _version: Version,
        _first_index: usize,
        _chunk_size: usize,
    ) -> Result<AccountStatesChunkWithProof> {
        unreachable!()
    }

    pub fn get_snapshot_receiver(
        self: &Arc<Self>,
        _version: Version,
        _expected_root_hash: HashValue,
    ) -> Result<Box<dyn StateSnapshotReceiver<AccountStateBlob>>> {
        unreachable!()
    }
}

/*
impl TreeReader<AccountStateBlob> for StateStore {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        self.db.get::<JellyfishMerkleNodeSchema>(node_key)
    }

    fn get_prev_node(
        &self,
        node_key: &NodeKey,
    ) -> Result<
        Option<(
            NodeKey,
            diem_jellyfish_merkle::node_type::Node<AccountStateBlob>,
        )>,
    > {
        let mut iter = self
            .db
            .iter::<JellyfishMerkleNodeSchema>(Default::default())?;
        iter.seek_for_prev(node_key)?;
        iter.next().transpose()
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>> {
        // Since everything has the same version during restore, we seek to the first node and get
        // its version.
        let mut iter = self
            .db
            .iter::<JellyfishMerkleNodeSchema>(Default::default())?;
        iter.seek_to_first();
        let version = match iter.next().transpose()? {
            Some((node_key, _node)) => node_key.version(),
            None => return Ok(None),
        };

        // The encoding of key and value in DB looks like:
        //
        // | <-------------- key --------------> | <- value -> |
        // | version | num_nibbles | nibble_path |    node     |
        //
        // Here version is fixed. For each num_nibbles, there could be a range of nibble paths
        // of the same length. If one of them is the rightmost leaf R, it must be at the end of this
        // range. Otherwise let's assume the R is in the middle of the range, so we
        // call the node at the end of this range X:
        //   1. If X is leaf, then X.account_key() > R.account_key(), because the nibble path is a
        //      prefix of the account key. So R is not the rightmost leaf.
        //   2. If X is internal node, then X must be on the right side of R, so all its children's
        //      account keys are larger than R.account_key(). So R is not the rightmost leaf.
        //
        // Given that num_nibbles ranges from 0 to ROOT_NIBBLE_HEIGHT, there are only
        // ROOT_NIBBLE_HEIGHT+1 ranges, so we can just find the node at the end of each range and
        // then pick the one with the largest account key.
        let mut ret = None;

        for num_nibbles in 1..=ROOT_NIBBLE_HEIGHT + 1 {
            let mut iter = self
                .db
                .iter::<JellyfishMerkleNodeSchema>(Default::default())?;
            // nibble_path is always non-empty except for the root, so if we use an empty nibble
            // path as the seek key, the iterator will end up pointing to the end of the previous
            // range.
            let seek_key = (version, num_nibbles as u8);
            iter.seek_for_prev(&seek_key)?;

            if let Some((node_key, node)) = iter.next().transpose()? {
                debug_assert_eq!(node_key.version(), version);
                debug_assert!(node_key.nibble_path().num_nibbles() < num_nibbles);

                if let Node::Leaf(leaf_node) = node {
                    match ret {
                        None => ret = Some((node_key, leaf_node)),
                        Some(ref other) => {
                            if leaf_node.account_key() > other.1.account_key() {
                                ret = Some((node_key, leaf_node));
                            }
                        }
                    }
                }
            }
        }

        Ok(ret)
    }
}

impl TreeWriter<AccountStateBlob> for StateStore {
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()> {
        let mut batch = SchemaBatch::new();
        add_node_batch(&mut batch, node_batch)?;
        self.db.write_schemas(batch)
    }
}
 */

/*
fn add_node_batch(batch: &mut SchemaBatch, node_batch: &NodeBatch) -> Result<()> {
    node_batch
        .iter()
        .map(|(node_key, node)| batch.put::<JellyfishMerkleNodeSchema>(node_key, node))
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
 */

fn swap_if<T>(first: T, second: T, cond: bool) -> (T, T) {
    if cond {
        (second, first)
    } else {
        (first, second)
    }
}
