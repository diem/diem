// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crypto::HashValue;
use schemadb::schema::assert_encode_decode;
use sparse_merkle::node_type::Node;

#[test]
fn test_state_merkle_node_schema() {
    assert_encode_decode::<StateMerkleNodeSchema>(
        &HashValue::random(),
        &Node::new_leaf(HashValue::random(), HashValue::random()),
    );
}
