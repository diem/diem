// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for LedgerInfoWithSignatures structure.
//!
//! Serialized LedgerInfoWithSignatures identified by version.
//! ```text
//! |<--key-->|<---------------value------------->|
//! | version | ledger_info_with_signatures bytes |
//! ```
//!
//! `Version` is serialized in big endian so that records in RocksDB will be in order of it's
//! numeric value.

use crate::schema::ensure_slice_len_eq;
use byteorder::{BigEndian, ReadBytesExt};
use failure::prelude::*;
use nextgen_crypto::ed25519::*;
use proto_conv::{FromProtoBytes, IntoProtoBytes};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    DEFAULT_CF_NAME,
};
use std::mem::size_of;
use types::{ledger_info::LedgerInfoWithSignatures, transaction::Version};

define_schema!(
    LedgerInfoSchema,
    Version,
    LedgerInfoWithSignatures<Ed25519Signature>,
    DEFAULT_CF_NAME
);

impl KeyCodec<LedgerInfoSchema> for Version {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Version>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<LedgerInfoSchema> for LedgerInfoWithSignatures<Ed25519Signature> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.clone().into_proto_bytes()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::from_proto_bytes(data)
    }
}

#[cfg(test)]
mod test;
