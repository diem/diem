// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    deserializer::load_signature_token,
    file_format::{SignatureToken, StructHandleIndex},
    file_format_common::{BinaryData, SIGNATURE_TOKEN_DEPTH_MAX},
    serializer::{serialize_signature_token, serialize_signature_token_unchecked},
};
use std::io::Cursor;

#[test]
fn serialize_and_deserialize_nested_types_max() {
    let mut ty = SignatureToken::Struct(StructHandleIndex::new(0));
    for _ in 1..SIGNATURE_TOKEN_DEPTH_MAX {
        ty = SignatureToken::Vector(Box::new(ty));
    }

    let mut binary = BinaryData::new();
    serialize_signature_token(&mut binary, &ty).expect("serialization should succeed");

    let mut cursor = Cursor::new(binary.as_inner());
    load_signature_token(&mut cursor).expect("deserialization should succeed");
}

#[test]
fn serialize_nested_types_too_deep() {
    let mut ty = SignatureToken::Struct(StructHandleIndex::new(0));
    for _ in 0..SIGNATURE_TOKEN_DEPTH_MAX {
        ty = SignatureToken::Vector(Box::new(ty));
    }

    let mut binary = BinaryData::new();
    serialize_signature_token(&mut binary, &ty).expect_err("serialization should fail");

    let mut binary = BinaryData::new();
    serialize_signature_token_unchecked(&mut binary, &ty)
        .expect("serialization (unchecked) should succeed");

    let mut cursor = Cursor::new(binary.as_inner());
    load_signature_token(&mut cursor).expect_err("deserialization should fail");
}
