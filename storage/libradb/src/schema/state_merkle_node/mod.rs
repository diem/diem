// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for nodes in state Merkle tree.
//! Node is identified by node_hash = Hash(serialized_node).
//! ```text
//! |<----key--->|<-----value----->|
//! |  node_hash | serialized_node |
//! ```

use crate::schema::STATE_MERKLE_NODE_CF_NAME;
use crypto::HashValue;
use failure::prelude::*;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use sparse_merkle::node_type::Node;

define_schema!(
    StateMerkleNodeSchema,
    HashValue,
    Node,
    STATE_MERKLE_NODE_CF_NAME
);

impl KeyCodec<StateMerkleNodeSchema> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(&data[..])?)
    }
}

impl ValueCodec<StateMerkleNodeSchema> for Node {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.encode()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(Node::decode(&data[..])?)
    }
}

#[cfg(test)]
mod test;
