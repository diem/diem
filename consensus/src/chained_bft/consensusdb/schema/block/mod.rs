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
use crate::chained_bft::{common::Payload, consensus_types::block::Block};
use crypto::HashValue;
use failure::prelude::*;
use prost::Message;
use prost_ext::MessageExt;
use schemadb::schema::{KeyCodec, Schema, ValueCodec};
use std::convert::TryInto;
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
        let block: network::proto::Block = self.borrow_into_block().clone().into();
        Ok(block.to_vec()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        let block: Block<T> = network::proto::Block::decode(data)?.try_into()?;
        Ok(Self::from_block(block))
    }
}

#[cfg(test)]
mod test;
