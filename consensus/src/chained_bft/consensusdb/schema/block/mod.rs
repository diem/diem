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
use consensus_types::{block::Block, common::Payload};
use libra_crypto::HashValue;
use schemadb::schema::{KeyCodec, Schema, ValueCodec};
use std::{cmp, fmt, marker::PhantomData};

pub struct BlockSchema<T: Payload> {
    phantom: PhantomData<T>,
}

impl<T: Payload> Schema for BlockSchema<T> {
    const COLUMN_FAMILY_NAME: schemadb::ColumnFamilyName = BLOCK_CF_NAME;
    type Key = HashValue;
    type Value = SchemaBlock<T>;
}

impl<T: Payload> KeyCodec<BlockSchema<T>> for HashValue {
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
pub struct SchemaBlock<T: Payload>(Block<T>);

impl<T: Payload> SchemaBlock<T> {
    pub fn from_block(sb: Block<T>) -> SchemaBlock<T> {
        Self(sb)
    }

    pub fn borrow_into_block(&self) -> &Block<T> {
        &self.0
    }
}

impl<T: Payload> cmp::PartialEq for SchemaBlock<T> {
    fn eq(&self, other: &SchemaBlock<T>) -> bool {
        self.borrow_into_block().eq(&other.borrow_into_block())
    }
}

impl<T: Payload> fmt::Debug for SchemaBlock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.borrow_into_block().fmt(f)
    }
}

impl<T: Payload> ValueCodec<BlockSchema<T>> for SchemaBlock<T> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(lcs::to_bytes(&self.0)?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(SchemaBlock(lcs::from_bytes(data)?))
    }
}

#[cfg(test)]
mod test;
