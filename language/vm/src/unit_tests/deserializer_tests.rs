// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{CompiledModule, CompiledScript},
    file_format_common::*,
};
use move_core_types::vm_status::StatusCode;

fn malformed_simple_versioned_test(version: u32) {
    // bad uleb (more than allowed for table count)
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(150); // table count (high bit 1)
    binary.push(150); // table count (high bit 1)
    binary.push(1);
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad uleb").major_status(),
        StatusCode::MALFORMED
    );

    // bad uleb (too big)
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
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
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(0); // table count
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected no table count").major_status(),
        StatusCode::MALFORMED
    );

    // missing tables
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(10); // table count
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected no table header").major_status(),
        StatusCode::MALFORMED
    );

    // missing table content
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
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
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
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
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(2); // table count
    binary.push(1); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    binary.push(2); // table type
    binary.push(100); // bad table offset
    binary.push(10); // table length
    binary.resize(binary.len() + 5000, 0);
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad table offset").major_status(),
        StatusCode::BAD_HEADER_TABLE
    );

    // incomplete table
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(1); // table count
    binary.push(1); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    binary.resize(binary.len() + 5, 0);
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad table content").major_status(),
        StatusCode::MALFORMED
    );

    // unknown table
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(1); // table count
    binary.push(100); // table type
    binary.push(0); // table offset
    binary.push(10); // table length
    binary.resize(binary.len() + 10, 0);
    let res = CompiledModule::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected unknown table").major_status(),
        StatusCode::UNKNOWN_TABLE_TYPE
    );

    // duplicate table
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
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
    binary.resize(binary.len() + 5000, 0);
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected table offset overflow")
            .major_status(),
        StatusCode::DUPLICATE_TABLE
    );

    // bad table in script
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&version.to_le_bytes()); // version
    binary.push(1); // table count
    binary.push(0xD); // table type - FieldHandle not good for script
    binary.push(0); // table offset
    binary.push(10); // table length
    binary.resize(binary.len() + 5000, 0);
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected table offset overflow")
            .major_status(),
        StatusCode::MALFORMED
    );
}

#[test]
#[allow(clippy::same_item_push)]
fn malformed_simple() {
    // empty binary
    let binary = vec![];
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::BAD_MAGIC
    );

    // under-sized binary
    let binary = vec![0u8, 0u8, 0u8];
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::BAD_MAGIC
    );

    // bad magic
    let binary = vec![0u8; 4];
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad magic").major_status(),
        StatusCode::BAD_MAGIC
    );

    // only magic
    let binary = BinaryConstants::DIEM_MAGIC.to_vec();
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::MALFORMED
    );

    // bad version
    let mut binary = BinaryConstants::DIEM_MAGIC.to_vec();
    binary.extend(&(VERSION_MAX.checked_add(1).unwrap()).to_le_bytes()); // version
    binary.push(10); // table count
    binary.push(0); // rest of binary
    let res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected unknown version").major_status(),
        StatusCode::UNKNOWN_VERSION
    );

    // versioned tests
    for version in &[VERSION_1, VERSION_2] {
        malformed_simple_versioned_test(*version);
    }
}

// Ensure that we can deserialize a script from disk
static EMPTY_SCRIPT: &[u8] = include_bytes!("../../../../types/src/test_helpers/empty_script.mv");

#[test]
fn deserialize_file() {
    CompiledScript::deserialize(EMPTY_SCRIPT).expect("script should deserialize properly");
}
