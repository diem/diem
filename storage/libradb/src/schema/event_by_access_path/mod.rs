// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for an event index via which a ContractEvent (
//! represented by a <txn_version, event_idx> tuple so that it can be fetched from `EventSchema`)
//! can be found by <access_path, sequence_num> tuple.
//!
//! ```text
//! |<----------key-------->|<----value---->|
//! | access_path | seq_num | txn_ver | idx |
//! ```

use crate::schema::{ensure_slice_len_eq, ensure_slice_len_gt, EVENT_BY_ACCESS_PATH_CF_NAME};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use failure::prelude::*;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::mem::size_of;
use types::{access_path::AccessPath, transaction::Version};

define_schema!(
    EventByAccessPathSchema,
    Key,
    Value,
    EVENT_BY_ACCESS_PATH_CF_NAME
);

type SeqNum = u64;
type Key = (AccessPath, SeqNum);

type Index = u64;
type Value = (Version, Index);

impl KeyCodec<EventByAccessPathSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (ref access_path, seq_num) = *self;

        let mut encoded = SimpleSerializer::<Vec<u8>>::serialize(access_path)?;
        encoded.write_u64::<BigEndian>(seq_num)?;

        Ok(encoded)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        let version_size = size_of::<Version>();
        ensure_slice_len_gt(data, version_size)?;
        let access_path_len = data.len() - version_size;

        let access_path = SimpleDeserializer::deserialize(&data[..access_path_len])?;
        let seq_num = (&data[access_path_len..]).read_u64::<BigEndian>()?;

        Ok((access_path, seq_num))
    }
}

impl ValueCodec<EventByAccessPathSchema> for Value {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let (version, index) = *self;

        let mut encoded = Vec::with_capacity(size_of::<Version>() + size_of::<Index>());
        encoded.write_u64::<BigEndian>(version)?;
        encoded.write_u64::<BigEndian>(index)?;

        Ok(encoded)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        let version_size = size_of::<Version>();

        let version = (&data[..version_size]).read_u64::<BigEndian>()?;
        let index = (&data[version_size..]).read_u64::<BigEndian>()?;
        Ok((version, index))
    }
}

#[cfg(test)]
mod test;
