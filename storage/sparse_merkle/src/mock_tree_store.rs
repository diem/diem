// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{node_type::Node, TreeReader, TreeUpdateBatch};
use crypto::HashValue;
use failure::prelude::*;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::RwLock,
};
use types::account_state_blob::AccountStateBlob;

#[derive(Default)]
pub(crate) struct MockTreeStore(
    RwLock<(
        HashMap<HashValue, Node>,
        HashMap<HashValue, AccountStateBlob>,
    )>,
);

impl TreeReader for MockTreeStore {
    fn get_node(&self, node_hash: HashValue) -> Result<Node> {
        Ok(self
            .0
            .read()
            .unwrap()
            .0
            .get(&node_hash)
            .cloned()
            .ok_or_else(|| format_err!("Failed to find node with hash {:?}", node_hash))?)
    }

    fn get_blob(&self, blob_hash: HashValue) -> Result<AccountStateBlob> {
        Ok(self
            .0
            .read()
            .unwrap()
            .1
            .get(&blob_hash)
            .cloned()
            .ok_or_else(|| format_err!("Failed to find blob with hash {:?}", blob_hash))?)
    }
}

impl MockTreeStore {
    pub fn put_node(&self, key: HashValue, node: Node) -> Result<()> {
        match self.0.write().unwrap().0.entry(key) {
            Entry::Occupied(_) => bail!("Key {:?} exists.", key),
            Entry::Vacant(v) => {
                v.insert(node);
            }
        }
        Ok(())
    }

    pub fn put_blob(&self, key: HashValue, blob: AccountStateBlob) -> Result<()> {
        self.0.write().unwrap().1.insert(key, blob);
        Ok(())
    }

    pub fn write_tree_update_batch(&self, batch: TreeUpdateBatch) -> Result<()> {
        let (node_batch, blob_batch) = batch.into();
        node_batch
            .into_iter()
            .map(|(k, v)| self.put_node(k, v))
            .collect::<Result<Vec<_>>>()?;
        blob_batch
            .into_iter()
            .map(|(k, v)| self.put_blob(k, v))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.0.read().unwrap().0.len()
    }

    pub fn num_blobs(&self) -> usize {
        self.0.read().unwrap().1.len()
    }
}
