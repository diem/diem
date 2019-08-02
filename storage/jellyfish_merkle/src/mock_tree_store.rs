// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    node_type::{Node, NodeKey},
    StaleNodeIndex, TreeReader, TreeUpdateBatch,
};
use failure::prelude::*;
use std::{
    collections::{hash_map::Entry, BTreeSet, HashMap},
    sync::RwLock,
};
use types::transaction::Version;

#[derive(Default)]
pub(crate) struct MockTreeStore(RwLock<(HashMap<NodeKey, Node>, BTreeSet<StaleNodeIndex>)>);

impl TreeReader for MockTreeStore {
    fn get_node(&self, node_key: &NodeKey) -> Result<Node> {
        Ok(self
            .0
            .read()
            .unwrap()
            .0
            .get(node_key)
            .cloned()
            .ok_or_else(|| format_err!("Failed to find node with hash {:?}", node_key))?)
    }
}

impl MockTreeStore {
    pub fn put_node(&self, node_key: NodeKey, node: Node) -> Result<()> {
        match self.0.write().unwrap().0.entry(node_key) {
            Entry::Occupied(o) => bail!("Key {:?} exists.", o.key()),
            Entry::Vacant(v) => {
                v.insert(node);
            }
        }
        Ok(())
    }

    fn put_stale_node_index(&self, index: StaleNodeIndex) -> Result<()> {
        let is_new_entry = self.0.write().unwrap().1.insert(index);
        ensure!(is_new_entry, "Duplicated retire log.");
        Ok(())
    }

    pub fn write_tree_update_batch(&self, batch: TreeUpdateBatch) -> Result<()> {
        let (node_batch, retired_record_batch) = batch.into();
        node_batch
            .into_iter()
            .map(|(k, v)| self.put_node(k, v))
            .collect::<Result<Vec<_>>>()?;
        retired_record_batch
            .into_iter()
            .map(|i| self.put_stale_node_index(i))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    pub fn purge_stale_nodes(&self, least_readable_version: Version) -> Result<()> {
        let mut wlocked = self.0.write().unwrap();

        // Only records retired before or at `least_readable_version` can be purged in order
        // to keep that version still readable.
        let to_prune = wlocked
            .1
            .iter()
            .take_while(|log| log.stale_since_version <= least_readable_version)
            .cloned()
            .collect::<Vec<_>>();

        for log in to_prune {
            let removed = wlocked.0.remove(&log.node_key).is_some();
            ensure!(removed, "Retire log refers to non-existent record.");
            wlocked.1.remove(&log);
        }

        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.0.read().unwrap().0.len()
    }
}
