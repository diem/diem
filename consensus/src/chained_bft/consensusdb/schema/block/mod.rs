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
use failure::prelude::*;
use libra_crypto::HashValue;
use libra_proto_conv::{FromProtoBytes, IntoProtoBytes};
use libra_schemadb::schema::{KeyCodec, Schema, ValueCodec};
use std::marker::PhantomData;

pub struct BlockSchema<T: Payload> {
    phantom: PhantomData<T>,
}

impl<T: Payload> Schema for BlockSchema<T> {
    const COLUMN_FAMILY_NAME: libra_schemadb::ColumnFamilyName = BLOCK_CF_NAME;
    type Key = HashValue;
    type Value = Block<T>;
}

impl<T: Payload> KeyCodec<BlockSchema<T>> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl<T: Payload> ValueCodec<BlockSchema<T>> for Block<T> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.clone().into_proto_bytes()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(Self::from_proto_bytes(data)?)
    }
}

#[cfg(test)]
mod test;
