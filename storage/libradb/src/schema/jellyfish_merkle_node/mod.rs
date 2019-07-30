// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for nodes in the state Jellyfish Merkle Tree.
//! Node is identified by [NodeKey](JellyfishMerkle::node_type::NodeKey).
//! ```text
//! |<----key--->|<-----value----->|
//! |  node_key  | serialized_node |
//! ```

use crate::schema::{ensure_slice_len_gt, JELLYFISH_MERKLE_NODE_CF_NAME};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use failure::prelude::*;
use jellyfish_merkle::node_type::{Node, NodeKey};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use sparse_merkle::nibble_path::NibblePath;
use std::{
    io::{Cursor, Write},
    mem::size_of,
};
use types::transaction::Version;

define_schema!(
    JellyfishMerkleNodeSchema,
    NodeKey,
    Node,
    JELLYFISH_MERKLE_NODE_CF_NAME
);

impl KeyCodec<JellyfishMerkleNodeSchema> for NodeKey {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let mut encoded_key = vec![];
        encoded_key.write_u64::<BigEndian>(self.version())?;
        encoded_key.write_u8((self.nibble_path().num_nibbles() & 1) as u8)?;
        encoded_key.write_all(self.nibble_path().bytes())?;
        Ok(encoded_key)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        let version_size = size_of::<Version>();
        let u8_size = size_of::<u8>();
        ensure_slice_len_gt(data, version_size + u8_size - 1)?;

        let mut reader = Cursor::new(data);
        let version = reader.read_u64::<BigEndian>()?;
        let is_odd = reader.read_u8()? == 1;
        let nibble_bytes = data[version_size + u8_size..].to_vec();
        let nibble_path = if is_odd {
            NibblePath::new_odd(nibble_bytes)
        } else {
            NibblePath::new(nibble_bytes)
        };
        Ok(NodeKey::new(version, nibble_path))
    }
}

impl ValueCodec<JellyfishMerkleNodeSchema> for Node {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.encode()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(Node::decode(&data[..])?)
    }
}

#[cfg(test)]
mod test;
