// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_lang::{move_compile, shared::Flags};

fn sanity_check_testsuite_impl(targets: Vec<String>, deps: Vec<String>) {
    let (files, units_or_errors) =
        move_compile(&targets, &deps, None, false, Flags::empty()).unwrap();
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
    sanity_check_testsuite_impl(diem_framework::diem_stdlib_files(), vec![])
}
