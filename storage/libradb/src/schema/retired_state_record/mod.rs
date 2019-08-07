// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the physical storage schema for information related to outdated account
//! state records and outdated state Merkle tree nodes, both of which are ready to be pruned
//! after being old enough.
//!
//! A record in this data set has 3 pieces of information:
//!     1. The version at which a retired record (in another data set) becomes retired, meaning,
//! replaced by an updated record.
//!     2. Which data set the retired record is in, either the account state data set or the sparse
//! Merkle nodes data set.
//!     3. The key to identify the retired record in the data set it belongs, which consists of the
//! version at which is was created and the hash of the record.
//!
//! ```text
//! |<----------------------key---------------------->|
//! | version_retired | type | version_created | hash |
//! ```
//!
//! `version_retired` is serialized in big endian so that records in RocksDB will be in order of its
//! numeric value.

use crate::schema::{ensure_slice_len_eq, RETIRED_STATE_RECORD_CF_NAME};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crypto::hash::HashValue;
use failure::prelude::*;
use num_traits::{FromPrimitive, ToPrimitive};
use schemadb::{
    define_schema,
    schema::{KeyCodec, SeekKeyCodec, ValueCodec},
};
use sparse_merkle::{RetiredRecordType, RetiredStateRecord};
use std::io::{Cursor, Write};
use types::transaction::Version;

define_schema!(
    RetiredStateRecordSchema,
    RetiredStateRecord,
    (),
    RETIRED_STATE_RECORD_CF_NAME
);

const ROW_LEN: usize = 49;

impl KeyCodec<RetiredStateRecordSchema> for RetiredStateRecord {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let mut encoded = Vec::with_capacity(ROW_LEN);
        encoded.write_u64::<BigEndian>(self.version_retired)?;
        encoded.write_u8(
            self.record_type
                .to_u8()
                .ok_or_else(|| format_err!("RecordType should be able to convert to u64."))?,
        )?;
        encoded.write_u64::<BigEndian>(self.version_created)?;
        encoded.write_all(self.hash.as_ref())?;

        Ok(encoded)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, ROW_LEN)?;
        let mut reader = Cursor::new(data);

        let version_retired = reader.read_u64::<BigEndian>()?;
        let record_type = RetiredRecordType::from_u8(reader.read_u8()?)
            .ok_or_else(|| format_err!("Failed to convert to RetiredRecordType."))?;
        let version_created = reader.read_u64::<BigEndian>()?;
        let hash = HashValue::from_slice(&data[(data.len() - HashValue::LENGTH)..])?;

        Ok(Self {
            version_retired,
            record_type,
            version_created,
            hash,
        })
    }
}

impl ValueCodec<RetiredStateRecordSchema> for () {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, 0)?;
        Ok(())
    }
}

impl SeekKeyCodec<RetiredStateRecordSchema> for Version {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }
}

#[cfg(test)]
mod test;
