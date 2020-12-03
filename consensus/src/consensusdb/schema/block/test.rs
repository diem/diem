// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use schemadb::schema::assert_encode_decode;

#[test]
fn test_encode_decode() {
    let block = Block::make_genesis_block();
    assert_encode_decode::<BlockSchema>(&block.id(), &block);
}
