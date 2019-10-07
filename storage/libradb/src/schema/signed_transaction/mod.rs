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

use crate::schema::{ensure_slice_len_eq, SIGNED_TRANSACTION_CF_NAME};
use byteorder::{BigEndian, ReadBytesExt};
use failure::prelude::*;
use libra_types::transaction::{SignedTransaction, Version};
use prost::Message;
use prost_ext::MessageExt;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::convert::TryInto;
use std::mem::size_of;

define_schema!(
    SignedTransactionSchema,
    Version,
    SignedTransaction,
    SIGNED_TRANSACTION_CF_NAME
);

impl KeyCodec<SignedTransactionSchema> for Version {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Version>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<SignedTransactionSchema> for SignedTransaction {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let event: libra_types::proto::types::SignedTransaction = self.clone().into();
        Ok(event.to_vec()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        libra_types::proto::types::SignedTransaction::decode(data)?.try_into()
    }
}

#[cfg(test)]
mod test;
