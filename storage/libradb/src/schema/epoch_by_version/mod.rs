// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for an index to help us find out which epoch a
//! ledger version is in, by storing a version <-> epoch pair for each version where the epoch
//! number bumps: a pair (`version`, `epoch_num`) indicates that the last version of `epoch_num` is
//! `version`.
//!
//! ```text
//! |<--key-->|<---value-->|
//! | version | epoch_num  |
//! ```
//!
//! `version` is serialized in big endian so that records in RocksDB will be in order of their
//! numeric value.

use crate::schema::{ensure_slice_len_eq, EPOCH_BY_VERSION_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use libra_types::transaction::Version;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::mem::size_of;

define_schema!(
    EpochByVersionSchema,
    Version,
    u64, // epoch_num
    EPOCH_BY_VERSION_CF_NAME
);

impl KeyCodec<EpochByVersionSchema> for Version {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<EpochByVersionSchema> for u64 {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

#[cfg(test)]
mod test;
