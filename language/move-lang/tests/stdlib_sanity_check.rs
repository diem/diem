// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_lang::{move_compile, shared::Address};
use move_lang_test_utils::*;

fn sanity_check_testsuite_impl(targets: Vec<String>, deps: Vec<String>) {
    let sender = Some(Address::DIEM_CORE);

    let (files, units_or_errors) = move_compile(&targets, &deps, sender, None, false).unwrap();
    let errors = match units_or_errors {
        Err(errors) => errors,
        Ok(units) => move_lang::compiled_unit::verify_units(units).1,
    };

    if !errors.is_empty() {
        let rendered_errors =
            String::from_utf8(move_lang::errors::report_errors_to_buffer(files, errors)).unwrap();

        panic!("Expected success. Unexpected errors:\n{}", rendered_errors);
    }
}

#[test]
fn test_stdlib_sanity_check() {
    sanity_check_testsuite_impl(
        diem_framework::script_files(),
        diem_framework::diem_stdlib_files(),
    );
    sanity_check_testsuite_impl(
        diem_framework::script_files(),
        vec![STD_LIB_COMPILED_DIR.to_string()],
    )
}
