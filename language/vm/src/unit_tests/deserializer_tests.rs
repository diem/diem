// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{CompiledModule, CompiledScript},
    file_format_common::*,
};
use move_core_types::vm_status::StatusCode;

#[test]
#[allow(clippy::same_item_push)]
fn malformed_simple() {
    // empty binary
    let mut binary = vec![];
    let mut res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::BAD_MAGIC
    );

    // under-sized binary
    binary = vec![0u8, 0u8, 0u8];
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::BAD_MAGIC
    );

    // bad magic
    binary = vec![0u8; 4];
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad magic").major_status(),
        StatusCode::BAD_MAGIC
    );

    // only magic
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::MALFORMED
    );

    // bad version
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(2); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(10); // table count
    binary.push(0); // rest of binary
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected unknown version").major_status(),
        StatusCode::UNKNOWN_VERSION
    );

    // bad version
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(2); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(10); // table count
    binary.push(0); // rest of binary
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected unknown version").major_status(),
        StatusCode::UNKNOWN_VERSION
    );

    // bad uleb (more than allowed for table count)
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(150); // table count (high bit 1)
    binary.push(150); // table count (high bit 1)
    binary.push(1);
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad uleb").major_status(),
        StatusCode::MALFORMED
    );

    // bad uleb (too big)
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(150); // table count (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(150); // table count again (high bit 1)
    binary.push(0); // table count again
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad uleb").major_status(),
        StatusCode::MALFORMED
    );

    // no tables
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(0); // table count
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected no table count").major_status(),
        StatusCode::MALFORMED
    );

    // missing tables
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(10); // table count
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected no table header").major_status(),
        StatusCode::MALFORMED
    );

    // missing table content
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(1); // table count
    binary.push(1); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected no table content").major_status(),
        StatusCode::MALFORMED
    );

    // bad table header (bad offset)
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(1); // table count
    binary.push(1); // table type
    binary.push(100); // bad table offset
    binary.push(10); // table length
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad table offset").major_status(),
        StatusCode::BAD_HEADER_TABLE
    );

    // bad table header (bad offset)
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(2); // table count
    binary.push(1); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    binary.push(2); // table type
    binary.push(100); // bad table offset
    binary.push(10); // table length
    for _ in 0..5000 {
        binary.push(0);
    }
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad table offset").major_status(),
        StatusCode::BAD_HEADER_TABLE
    );

    // incomplete table
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(1); // table count
    binary.push(1); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    for _ in 0..5 {
        binary.push(0);
    }
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad table content").major_status(),
        StatusCode::MALFORMED
    );

    // unknown table
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(1); // table count
    binary.push(100); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    for _ in 0..10 {
        binary.push(0);
    }
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected unknown table").major_status(),
        StatusCode::UNKNOWN_TABLE_TYPE
    );

    // duplicate table
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(3); // table count
    binary.push(1); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    binary.push(2); // table type
    binary.push(10); // table offset
    binary.push(10); // table length
    binary.push(1); // table type
    binary.push(20); // table offset
    binary.push(10); // table length
    for _ in 0..5000 {
        binary.push(0);
    }
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected table offset overflow")
            .major_status(),
        StatusCode::DUPLICATE_TABLE
    );

    // bad table in script
    binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.push(1); // version
    binary.push(0);
    binary.push(0);
    binary.push(0);
    binary.push(1); // table count
    binary.push(0xD); // table type - FieldHandle not good for script
    binary.push(0); // table offset
    binary.push(10); // table length
    for _ in 0..5000 {
        binary.push(0);
    }
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected table offset overflow")
            .major_status(),
        StatusCode::MALFORMED
    );
}

// Ensure that we can deserialize a script from disk
static EMPTY_SCRIPT: &[u8] = include_bytes!("../../../../types/src/test_helpers/empty_script.mv");

#[test]
fn deserialize_file() {
    CompiledScript::deserialize(EMPTY_SCRIPT).expect("script should deserialize properly");
}
