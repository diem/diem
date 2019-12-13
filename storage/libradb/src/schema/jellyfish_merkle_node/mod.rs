// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for nodes in the state Jellyfish Merkle Tree.
//! Node is identified by [NodeKey](jellyfish-merkle::node_type::NodeKey).
//! ```text
//! |<----key--->|<-----value----->|
//! |  node_key  | serialized_node |
//! ```

use crate::schema::JELLYFISH_MERKLE_NODE_CF_NAME;
use anyhow::Result;
use jellyfish_merkle::node_type::{Node, NodeKey};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};

define_schema!(
    JellyfishMerkleNodeSchema,
    NodeKey,
    Node,
    JELLYFISH_MERKLE_NODE_CF_NAME
);

impl KeyCodec<JellyfishMerkleNodeSchema> for NodeKey {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<JellyfishMerkleNodeSchema> for Node {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.encode()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(Self::decode(&data[..])?)
    }
}

#[cfg(test)]
mod test;
