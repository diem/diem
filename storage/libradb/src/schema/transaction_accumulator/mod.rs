// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for the transaction accumulator.
//!
//! A hash value is stored on each position.
//! See `storage/accumulator/lib.rs` for details.
//! ```text
//! |<----------key--------->|<-value->|
//! | position in post order |   hash  |
//! ```

use crate::schema::{ensure_slice_len_eq, TRANSACTION_ACCUMULATOR_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use libra_crypto::HashValue;
use libra_types::proof::position::Position;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::mem::size_of;

define_schema!(
    TransactionAccumulatorSchema,
    Position,
    HashValue,
    TRANSACTION_ACCUMULATOR_CF_NAME
);

impl KeyCodec<TransactionAccumulatorSchema> for Position {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_postorder_index().to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<u64>())?;
        Ok(Position::from_postorder_index(
            (&data[..]).read_u64::<BigEndian>()?,
        ))
    }
}

impl ValueCodec<TransactionAccumulatorSchema> for HashValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::from_slice(data)
    }
}

#[cfg(test)]
mod test;
