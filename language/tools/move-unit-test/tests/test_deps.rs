// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};
use move_unit_test::{self, UnitTestingConfig};
use std::path::PathBuf;

// Make sure the compiled bytecode for dependencies is included, but the tests in them are not run.
#[test]
fn test_deps_arent_tested() {
    let mut testing_config = UnitTestingConfig::default_with_bound(None)
        .with_named_addresses(move_stdlib::move_stdlib_named_addresses());
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let a_path = path.join("tests/sources/A.move");
    let b_path = path.join("tests/sources/B.move");

    testing_config.source_files = vec![b_path.to_str().unwrap().to_owned()];
    testing_config.dep_files = vec![a_path.to_str().unwrap().to_owned()];

    let test_plan = testing_config.build_test_plan().unwrap();

    let mut iter = test_plan.module_tests.into_iter();
    let (mod_id, _) = iter.next().unwrap();
    let expected_mod_id = ModuleId::new(
        AccountAddress::from_hex_literal("0x1").unwrap(),
        Identifier::new("B").unwrap(),
    );
    assert!(mod_id == expected_mod_id);
    assert!(iter.next().is_none());
}
