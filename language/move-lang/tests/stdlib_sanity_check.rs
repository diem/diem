// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_lang::{move_check_no_report, shared::Address};
use std::{fs, path::Path};

use move_lang::test_utils::*;

const OUT_EXT: &str = "out";

const KEEP_TMP: &str = "KEEP";

// Runs all tests under the test/testsuite directory.
fn sanity_check_testsuite(path: &Path) -> datatest_stable::Result<()> {
    let mut targets: Vec<&'static str> =
        vec![Box::leak(Box::new(path.to_str().unwrap().to_owned()))];
    targets.append(&mut stdlib_files());
    let sender = Some(Address::parse_str(SENDER).unwrap());

    let out_path = path.with_extension(OUT_EXT);

    let (files, errors) = move_check_no_report(&targets, &[], sender)?;
    let has_errors = !errors.is_empty();
    let error_buffer = if has_errors {
        move_lang::errors::report_errors_to_buffer(files, errors)
    } else {
        vec![]
    };

    let save_errors = read_bool_var(KEEP_TMP);

    fs::write(out_path.clone(), error_buffer)?;
    let rendered_errors = fs::read_to_string(out_path.clone())?;
    if !save_errors {
        fs::remove_file(out_path)?;
    }

    if has_errors {
        let msg = format!("Expected success. Unexpected errors:\n{}", rendered_errors);
        error(msg)
    } else {
        Ok(())
    }
}

datatest_stable::harness!(
    sanity_check_testsuite,
    "stdlib/transaction_scripts",
    r".*\.move"
);
