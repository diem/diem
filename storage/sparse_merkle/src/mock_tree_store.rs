// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{node_type::Node, RetiredRecordType, RetiredStateRecord, TreeReader, TreeUpdateBatch};
use crypto::HashValue;
use failure::prelude::*;
use std::{
    collections::{hash_map::Entry, BTreeSet, HashMap},
    sync::RwLock,
};
use types::{account_state_blob::AccountStateBlob, transaction::Version};

#[derive(Default)]
pub(crate) struct MockTreeStore(
    RwLock<(
        HashMap<HashValue, Node>,
        HashMap<HashValue, AccountStateBlob>,
        BTreeSet<RetiredStateRecord>,
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

    fn put_blob(&self, key: HashValue, blob: AccountStateBlob) -> Result<()> {
        self.0.write().unwrap().1.insert(key, blob);
        Ok(())
    }

    fn put_retired_record(&self, record: RetiredStateRecord) -> Result<()> {
        let is_new_entry = self.0.write().unwrap().2.insert(record);
        ensure!(is_new_entry, "Duplicated retire log.");
        Ok(())
    }

    pub fn write_tree_update_batch(&self, batch: TreeUpdateBatch) -> Result<()> {
        let (node_batch, blob_batch, retired_record_batch) = batch.into();
        node_batch
            .into_iter()
            .map(|(k, v)| self.put_node(k, v))
            .collect::<Result<Vec<_>>>()?;
        blob_batch
            .into_iter()
            .map(|(k, v)| self.put_blob(k, v))
            .collect::<Result<Vec<_>>>()?;
        retired_record_batch
            .into_iter()
            .map(|r| self.put_retired_record(r))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    pub fn purge_retired_records(&self, least_readable_version: Version) -> Result<()> {
        let mut wlocked = self.0.write().unwrap();

        // Only records retired before or at `least_readable_version` can be purged in order
        // to keep that version still readable.
        let to_prune = wlocked
            .2
            .iter()
            .take_while(|log| log.version_retired <= least_readable_version)
            .cloned()
            .collect::<Vec<_>>();

        for log in to_prune {
            let removed = match log.record_type {
                RetiredRecordType::Node => wlocked.0.remove(&log.hash).is_some(),
            };
            ensure!(removed, "Retire log refers to non-existent record.");
            wlocked.2.remove(&log);
        }

        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.0.read().unwrap().0.len()
    }

    pub fn num_blobs(&self) -> usize {
        self.0.read().unwrap().1.len()
    }
}
