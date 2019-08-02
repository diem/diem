// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

#[cfg(test)]
mod mock_tree_store;
mod node_type;
mod tree_cache;

use crypto::HashValue;
use failure::prelude::*;
use node_type::{Node, NodeKey};
use std::collections::{HashMap, HashSet};
use types::transaction::Version;

/// The hardcoded maximum height of a [`JellyfishMerkleTree`] in nibbles.
const ROOT_NIBBLE_HEIGHT: usize = HashValue::LENGTH * 2;

/// `TreeReader` defines the interface between [`JellyfishMerkleTree`] and underlying storage
/// holding nodes.
pub trait TreeReader {
    /// Get node given a node key.
    fn get_node(&self, node_key: &NodeKey) -> Result<Node>;
}

/// Node batch that will be written into db atomically with other batches.
pub type NodeBatch = HashMap<NodeKey, Node>;
/// [`RetireNodeIndex`] batch that will be written into db atomically with other batches.
pub type StaleNodeIndexBatch = HashSet<StaleNodeIndex>;

/// Indicates a node becomes stale since `stale_since_version`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaleNodeIndex {
    /// The version since when the node is overwritten and becomes stale.
    pub stale_since_version: Version,
    /// The [`NodeKey`](node_type::NodeKey) identifying the node associated with this
    /// record.
    pub node_key: NodeKey,
}

/// This is a wrapper of [`NodeBatch`] and [`StaleNodeIndexBatch`] that represents the incremental
/// updates of a tree and pruning indices after applying a write set, which is a vector of
/// `hashed_account_address` and `new_account_state_blob` pairs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeUpdateBatch {
    node_batch: NodeBatch,
    stale_node_index_batch: StaleNodeIndexBatch,
}

/// Conversion between tuple type and [`TreeUpdateBatch`].
impl From<TreeUpdateBatch> for (NodeBatch, StaleNodeIndexBatch) {
    fn from(batch: TreeUpdateBatch) -> Self {
        (batch.node_batch, batch.stale_node_index_batch)
    }
}
