// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::schema::block_index::BlockIndexSchema;
use failure::prelude::*;
use libra_types::block_index::BlockIndex;
use schemadb::{ReadOptions, SchemaBatch, DB};
use std::sync::Arc;

pub(crate) struct BlockIndexStore {
    db: Arc<DB>,
}

impl BlockIndexStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Insert BlockIndex
    pub fn insert_block_index(&self, height: &u64, block_index: &BlockIndex) -> Result<()> {
        let mut batch = SchemaBatch::new();
        batch.put::<BlockIndexSchema>(&height, &block_index)?;
        self.db.write_schemas(batch)
    }

    /// Load BlockIndex
    pub fn _load_block_index(&self) -> Result<Vec<BlockIndex>> {
        unimplemented!()
    }

    pub fn query_block_index(&self, height: Option<u64>, size: usize) -> Result<Vec<BlockIndex>> {
        let mut block_index_list = vec![];
        let mut begin = match height {
            Some(h) => h,
            None => {
                let mut iter = self.db.iter::<BlockIndexSchema>(ReadOptions::default())?;
                iter.seek_to_last();
                let result = iter.next();
                match result {
                    Some(val) => {
                        let (k, _v) = val.expect("Get value from db err.");
                        k
                    }
                    None => 0, //todo:err
                }
            }
        };

        while block_index_list.len() < size {
            let block_index: Option<BlockIndex> = self.db.get::<BlockIndexSchema>(&begin)?;
            match block_index {
                Some(index) => {
                    block_index_list.push(index);
                    begin = begin - 1;
                }
                None => {
                    break;
                }
            }
        }
        block_index_list.reverse();
        Ok(block_index_list)
    }
}
