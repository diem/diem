// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_types::{event::EventKey, transaction::Version};
use std::{
    collections::{hash_map, HashMap},
    sync::Arc,
};
use storage_interface::DbReader;

pub(crate) struct SequenceNumberAssigner {
    db_reader: Arc<dyn DbReader>,
    block_start_version: Version,
    assigned: HashMap<EventKey, u64>,
}

impl SequenceNumberAssigner {
    pub fn new(db_reader: Arc<dyn DbReader>, block_start_version: Version) -> Self {
        Self {
            db_reader,
            block_start_version,
            assigned: Default::default(),
        }
    }

    pub fn assign(&mut self, key: &EventKey) -> Result<u64> {
        let ret = match self.assigned.entry(*key) {
            hash_map::Entry::Occupied(mut entry) => {
                let assigned = entry.get_mut();
                *assigned += 1;
                *assigned
            }
            hash_map::Entry::Vacant(entry) => {
                let next = if self.block_start_version == 0 {
                    0
                } else {
                    self.db_reader
                        .get_next_sequence_number(self.block_start_version - 1, key)?
                };
                *entry.insert(next)
            }
        };

        Ok(ret)
    }
}
