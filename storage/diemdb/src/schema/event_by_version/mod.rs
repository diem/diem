// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for an event index via which a ContractEvent (
//! represented by a <txn_version, event_idx> tuple so that it can be fetched from `EventSchema`)
//! can be found by <access_path, version, sequence_num> tuple.
//!
//! ```text
//! |<--------------key------------>|<-value->|
//! | event_key | txn_ver | seq_num |   idx   |
//! ```

use crate::schema::{ensure_slice_len_eq, EVENT_BY_VERSION_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use diem_types::{event::EventKey, transaction::Version};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::{convert::TryFrom, mem::size_of};

define_schema!(EventByVersionSchema, Key, Value, EVENT_BY_VERSION_CF_NAME);

type SeqNum = u64;
type Key = (EventKey, Version, SeqNum);

type Index = u64;
type Value = Index;

impl KeyCodec<EventByVersionSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (ref event_key, version, seq_num) = *self;

        let mut encoded = event_key.to_vec();
        encoded.write_u64::<BigEndian>(version)?;
        encoded.write_u64::<BigEndian>(seq_num)?;

        Ok(encoded)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        const EVENT_KEY_LEN: usize = size_of::<EventKey>();
        const EVENT_KEY_AND_VER_LEN: usize = size_of::<(EventKey, Version)>();
        let event_key = EventKey::try_from(&data[..EVENT_KEY_LEN])?;
        let version = (&data[EVENT_KEY_LEN..]).read_u64::<BigEndian>()?;
        let seq_num = (&data[EVENT_KEY_AND_VER_LEN..]).read_u64::<BigEndian>()?;

        Ok((event_key, version, seq_num))
    }
}

impl ValueCodec<EventByVersionSchema> for Value {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_value(mut data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        Ok(data.read_u64::<BigEndian>()?)
    }
}

#[cfg(test)]
mod test;
