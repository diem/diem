// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use schemadb::schema::assert_encode_decode;

#[test]
fn test_encode_decode() {
    let genesis_block = SchemaBlock::from_block(Block::make_genesis_block());
    assert_encode_decode::<BlockSchema>(&HashValue::random(), &genesis_block);
}
