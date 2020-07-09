// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{CompiledModule, CompiledScript},
    file_format_common::*,
};
use move_core_types::vm_status::StatusCode;

#[test]
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
    binary = vec![0u8; 15];
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected bad magic").major_status(),
        StatusCode::BAD_MAGIC
    );

    // only magic
    binary = BinaryConstants::LIBRA_MAGIC.to_vec();
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected malformed binary").major_status(),
        StatusCode::MALFORMED
    );

    // bad major version
    binary = BinaryConstants::LIBRA_MAGIC.to_vec();
    binary.push(2); // major version
    binary.push(0); // minor version
    binary.push(10); // table count
    binary.push(0); // rest of binary ;)
    res = CompiledScript::deserialize(&binary);
    assert_eq!(
        res.expect_err("Expected unknown version").major_status(),
        StatusCode::UNKNOWN_VERSION
    );

    // bad minor version
    binary = BinaryConstants::LIBRA_MAGIC.to_vec();
    binary.push(1); // major version
    binary.push(1); // minor version
    binary.push(10); // table count
    binary.push(0); // rest of binary ;)
    let res1 = CompiledModule::deserialize(&binary);
    assert_eq!(
        res1.expect_err("Expected unknown version").major_status(),
        StatusCode::UNKNOWN_VERSION
    );
}

// Ensure that we can deserialize a script from disk
static EMPTY_SCRIPT: &[u8] = include_bytes!("../../../../types/src/test_helpers/empty_script.mv");

#[test]
fn deserialize_file() {
    CompiledScript::deserialize(EMPTY_SCRIPT).expect("script should deserialize properly");
}
