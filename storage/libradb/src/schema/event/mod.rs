// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for the contract events.
//!
//! An event is keyed by the version of the transaction it belongs to and the index of it among all
//! events yielded by the same transaction.
//! ```text
//! |<-------key----->|<---value--->|
//! | version | index | event bytes |
//! ```

use crate::schema::{ensure_slice_len_eq, EVENT_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libra_prost_ext::MessageExt;
use libra_types::{contract_event::ContractEvent, transaction::Version};
use prost::Message;
use schemadb::{
    define_schema,
    schema::{KeyCodec, SeekKeyCodec, ValueCodec},
};
use std::convert::TryInto;
use std::mem::size_of;

define_schema!(EventSchema, Key, ContractEvent, EVENT_CF_NAME);

type Index = u64;
type Key = (Version, Index);

impl KeyCodec<EventSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (version, index) = *self;

        let mut encoded_key = Vec::with_capacity(size_of::<Version>() + size_of::<Index>());
        encoded_key.write_u64::<BigEndian>(version)?;
        encoded_key.write_u64::<BigEndian>(index)?;
        Ok(encoded_key)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        let version_size = size_of::<Version>();

        let version = (&data[..version_size]).read_u64::<BigEndian>()?;
        let index = (&data[version_size..]).read_u64::<BigEndian>()?;
        Ok((version, index))
    }
}

impl ValueCodec<EventSchema> for ContractEvent {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let event: libra_types::proto::types::Event = self.clone().into();
        Ok(event.to_vec()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        libra_types::proto::types::Event::decode(data)?.try_into()
    }
}

impl SeekKeyCodec<EventSchema> for Version {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }
}

#[cfg(test)]
mod test;
