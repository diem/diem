// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for the event accumulator.
//!
//! Each version has its own event accumulator and a hash value is stored on each position within an
//! accumulator. See `storage/accumulator/lib.rs` for details.
//! ```text
//! |<--------key------->|<-value->|
//! | version | position |  hash   |
//! ```

use crate::schema::{ensure_slice_len_eq, EVENT_ACCUMULATOR_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libra_crypto::hash::HashValue;
use libra_types::{proof::position::Position, transaction::Version};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::mem::size_of;

define_schema!(
    EventAccumulatorSchema,
    Key,
    HashValue,
    EVENT_ACCUMULATOR_CF_NAME
);

type Key = (Version, Position);

impl KeyCodec<EventAccumulatorSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (version, position) = self;

        let mut encoded_key = Vec::with_capacity(size_of::<Version>() + size_of::<u64>());
        encoded_key.write_u64::<BigEndian>(*version)?;
        encoded_key.write_u64::<BigEndian>(position.to_inorder_index())?;
        Ok(encoded_key)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        let version_size = size_of::<Version>();

        let version = (&data[..version_size]).read_u64::<BigEndian>()?;
        let position = (&data[version_size..]).read_u64::<BigEndian>()?;
        Ok((version, Position::from_inorder_index(position)))
    }
}

impl ValueCodec<EventAccumulatorSchema> for HashValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::from_slice(data)
    }
}

#[cfg(test)]
mod test;
