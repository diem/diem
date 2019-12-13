// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for LedgerInfoWithSignatures structure.
//!
//! Serialized LedgerInfoWithSignatures identified by `epoch`.
//! ```text
//! |<---key--->|<---------------value------------->|
//! | epoch | ledger_info_with_signatures bytes |
//! ```
//!
//! `epoch` is serialized in big endian so that records in RocksDB will be in order of their
//! numeric value.

use crate::schema::ensure_slice_len_eq;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use libra_prost_ext::MessageExt;
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use prost::Message;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    DEFAULT_CF_NAME,
};
use std::convert::TryInto;
use std::mem::size_of;

define_schema!(
    LedgerInfoSchema,
    u64, /* epoch num */
    LedgerInfoWithSignatures,
    DEFAULT_CF_NAME
);

impl KeyCodec<LedgerInfoSchema> for u64 {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<LedgerInfoSchema> for LedgerInfoWithSignatures {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let event: libra_types::proto::types::LedgerInfoWithSignatures = self.clone().into();
        Ok(event.to_vec()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        libra_types::proto::types::LedgerInfoWithSignatures::decode(data)?.try_into()
    }
}

#[cfg(test)]
mod test;
