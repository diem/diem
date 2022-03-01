// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the physical storage schema for information related to outdated state
//! Jellyfish Merkle tree nodes, which are ready to be pruned after being old enough.
//!
//! An index entry in this data set has 2 pieces of information:
//!     1. The version since which a node (in another data set) becomes stale, meaning,
//! replaced by an updated node.
//!     2. The node_key to identify the stale node.
//!
//! ```text
//! |<--------------key-------------->|
//! | stale_since_vesrion | node_key |
//! ```
//!
//! `stale_since_version` is serialized in big endian so that records in RocksDB will be in order of
//! its numeric value.

use crate::schema::{ensure_slice_len_eq, BSMT_UPDATE_LOG_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use diem_crypto::HashValue;
use diem_types::{transaction::Version, vec_bool::VecBool};
use schemadb::{
    define_schema,
    schema::{KeyCodec, SeekKeyCodec, ValueCodec},
};
use serde::{Deserialize, Serialize};
use std::{io::Write, mem::size_of};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum BsmtKey {
    Node(VecBool),
    Value(HashValue),
}

type Key = (Version, BsmtKey);

define_schema!(BsmtUpdateLogSchema, Key, (), BSMT_UPDATE_LOG_CF_NAME);

impl KeyCodec<BsmtUpdateLogSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (ver, key) = self;

        let mut encoded = vec![];
        encoded.write_u64::<BigEndian>(!*ver)?;
        encoded.write_all(&bcs::to_bytes(key)?)?;

        Ok(encoded)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        const VERSION_SIZE: usize = size_of::<Version>();

        let ver = !(&data[..VERSION_SIZE]).read_u64::<BigEndian>()?;
        let key = bcs::from_bytes(&data[VERSION_SIZE..])?;

        Ok((ver, key))
    }
}

impl ValueCodec<BsmtUpdateLogSchema> for () {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, 0)?;
        Ok(())
    }
}

impl SeekKeyCodec<BsmtUpdateLogSchema> for Version {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        Ok((!*self).to_be_bytes().to_vec())
    }
}

#[cfg(test)]
mod test;
