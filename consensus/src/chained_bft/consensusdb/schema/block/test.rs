// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_schemadb::schema::assert_encode_decode;

#[test]
fn test_encode_decode() {
    assert_encode_decode::<BlockSchema<i64>>(&HashValue::random(), &Block::make_genesis_block());
}
