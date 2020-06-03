// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for consensus block.
//!
//! Serialized block bytes identified by block_hash.
//! ```text
//! |<---key---->|<---value--->|
//! | block_hash |    block    |
//! ```

use super::BLOCK_CF_NAME;
use anyhow::Result;
use consensus_types::block::Block;
use libra_crypto::HashValue;
use schemadb::schema::{KeyCodec, Schema, ValueCodec};
use std::{cmp, fmt};

pub struct BlockSchema;

impl Schema for BlockSchema {
    const COLUMN_FAMILY_NAME: schemadb::ColumnFamilyName = BLOCK_CF_NAME;
    type Key = HashValue;
    type Value = SchemaBlock;
}

impl KeyCodec<BlockSchema> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

#[derive(Clone)]
/// SchemaBlock is a crate wrapper for Block that is defined outside this crate.
/// ValueCodec cannot be implemented for Block here as Block is defined in
/// consensus_types crate (E0210).
pub struct SchemaBlock(Block);

impl SchemaBlock {
    pub fn from_block(sb: Block) -> SchemaBlock {
        Self(sb)
    }

    pub fn borrow_into_block(&self) -> &Block {
        &self.0
    }
}

impl cmp::PartialEq for SchemaBlock {
    fn eq(&self, other: &SchemaBlock) -> bool {
        self.borrow_into_block().eq(&other.borrow_into_block())
    }
}

impl fmt::Debug for SchemaBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.borrow_into_block().fmt(f)
    }
}

impl ValueCodec<BlockSchema> for SchemaBlock {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(lcs::to_bytes(&self.0)?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(SchemaBlock(lcs::from_bytes(data)?))
    }
}

#[cfg(test)]
mod test;
