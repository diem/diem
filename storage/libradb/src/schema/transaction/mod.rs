// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for signed transactions.
//!
//! Serialized signed transaction bytes identified by version.
//! ```text
//! |<--key-->|<--value-->|
//! | version | txn bytes |
//! ```
//!
//! `Version` is serialized in big endian so that records in RocksDB will be in order of it's
//! numeric value.

use crate::schema::{ensure_slice_len_eq, TRANSACTION_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use libra_types::transaction::{Transaction, Version};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::mem::size_of;

define_schema!(TransactionSchema, Version, Transaction, TRANSACTION_CF_NAME);

impl KeyCodec<TransactionSchema> for Version {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Version>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<TransactionSchema> for Transaction {
    fn encode_value(&self) -> Result<Vec<u8>> {
        lcs::to_bytes(self).map_err(Into::into)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        lcs::from_bytes(data).map_err(Into::into)
    }
}

#[cfg(test)]
mod test;
