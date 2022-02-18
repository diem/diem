// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// FIXME(aldenhu): update doc
//! This module defines physical storage schema for nodes in the state Jellyfish Merkle Tree.
//! Node is identified by [NodeKey](jellyfish-merkle::node_type::NodeKey).
//! ```text
//! |<----key--->|<-----value----->|
//! |  node_key  | serialized_node |
//! ```

use crate::schema::STATE_TREE_CF_NAME;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use diem_types::{
    transaction::{StateTreeNode, Version},
    vec_bool::VecBool,
};
use schemadb::{
    define_schema,
    schema::{KeyCodec, SeekKeyCodec, ValueCodec},
};

type Key = (VecBool, Version);

define_schema!(StateTreeSchema, Key, StateTreeNode, STATE_TREE_CF_NAME);

impl KeyCodec<StateTreeSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        // FIXME(aldenhu): more efficient encoding
        let mut data = Vec::new();

        let (pos, ver) = self;

        data.write_u16::<BigEndian>(pos.len() as u16)?;
        data.extend_from_slice(pos.bytes());
        data.write_u64::<BigEndian>(!*ver)?;

        Ok(data)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        let len = (&data[0..2]).read_u16::<BigEndian>()? as usize;
        let nbytes = len / 8 + if len % 8 == 0 { 0 } else { 1 };
        let bytes = (&data[2..2 + nbytes]).to_vec();
        let ver = !(&data[2 + nbytes..]).read_u64::<BigEndian>()?;

        let pos = VecBool::new_hack(len, bytes);

        Ok((pos, ver))
    }
}

impl ValueCodec<StateTreeSchema> for StateTreeNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(bcs::to_bytes(self)?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(data)?)
    }
}

impl SeekKeyCodec<StateTreeSchema> for VecBool {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        Ok(bcs::to_bytes(self)?)
    }
}

/*
#[cfg(test)]
mod test;
 */
