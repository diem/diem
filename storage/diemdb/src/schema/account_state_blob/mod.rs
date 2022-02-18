// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// FIXME(aldehu): update
//! This module defines physical storage schema for nodes in the state Jellyfish Merkle Tree.
//! Node is identified by [NodeKey](jellyfish-merkle::node_type::NodeKey).
//! ```text
//! |<----key--->|<-----value----->|
//! |  node_key  | serialized_node |
//! ```

use crate::schema::ACCOUNT_STATE_BLOB_CF_NAME;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use diem_crypto::HashValue;
use diem_types::{account_state_blob::AccountStateBlob, transaction::Version};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};

type Key = (HashValue, Version);

type Value = AccountStateBlob;

define_schema!(
    AccountStateBlobSchema,
    Key,
    Value,
    ACCOUNT_STATE_BLOB_CF_NAME
);

impl KeyCodec<AccountStateBlobSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (key, ver) = self;
        let mut data = key.to_vec();

        data.write_u64::<BigEndian>(!*ver)?;

        Ok(data)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        let key = HashValue::from_slice(&data[0..HashValue::LENGTH])?;
        let ver = !(&data[HashValue::LENGTH..]).read_u64::<BigEndian>()?;

        Ok((key, ver))
    }
}

impl ValueCodec<AccountStateBlobSchema> for Value {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(bcs::to_bytes(self)?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(data)?)
    }
}

// FIXME(aldenhu): remove
/*
impl SeekKeyCodec<AccountStateBlobSchema> for (Version, u8) {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(size_of::<Version>() + size_of::<u8>());
        out.write_u64::<BigEndian>(self.0)?;
        out.write_u8(self.1)?;
        Ok(out)
    }
}

#[cfg(test)]
mod test;
 */
