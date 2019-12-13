// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for system counters associated with ledger versions.
//!
//! Each version has values for multiple counters and they are serialized as a map.
//! ```text
//! |<--key-->|<--value->|
//! | version | counters |
//! ```
//!
//! `Version` is serialized in big endian so that records in RocksDB will be in order of it's
//! numeric value.

use super::LEDGER_COUNTERS_CF_NAME;
use crate::{ledger_counters::LedgerCounters, schema::ensure_slice_len_eq};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use libra_types::transaction::Version;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::mem::size_of;

define_schema!(
    LedgerCountersSchema,
    Version,
    LedgerCounters,
    LEDGER_COUNTERS_CF_NAME
);

impl KeyCodec<LedgerCountersSchema> for Version {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Version>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<LedgerCountersSchema> for LedgerCounters {
    fn encode_value(&self) -> Result<Vec<u8>> {
        lcs::to_bytes(self).map_err(Into::into)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        lcs::from_bytes(data).map_err(Into::into)
    }
}

#[cfg(test)]
mod test;
