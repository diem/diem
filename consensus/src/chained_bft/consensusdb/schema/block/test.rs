// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use nextgen_crypto::ed25519::*;
use schemadb::schema::assert_encode_decode;

#[test]
fn test_encode_decode() {
    assert_encode_decode::<BlockSchema<i64, Ed25519Signature>>(
        &HashValue::random(),
        &Block::make_genesis_block(),
    );
}
