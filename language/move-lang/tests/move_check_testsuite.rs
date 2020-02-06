// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use difference;
use move_lang::{move_compile_no_report, shared::Address};
use std::{fs, path::Path};

use move_lang::test_utils::*;

const OUT_EXT: &str = "out";
const EXP_EXT: &str = "exp";

const KEEP_TMP: &str = "KEEP";
const UPDATE_BASELINE: &str = "UPDATE_BASELINE";

fn format_diff(expected: String, actual: String) -> String {
    use difference::*;

    let changeset = Changeset::new(&expected, &actual, "\n");

    let mut ret = String::new();

    for seq in changeset.diffs {
        match &seq {
            Difference::Same(x) => {
                ret.push_str(x);
                ret.push_str("\n");
            }
            Difference::Add(x) => {
                ret.push_str("\x1B[92m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push_str("\n");
            }
            Difference::Rem(x) => {
                ret.push_str("\x1B[91m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push_str("\n");
            }
        }
    }
    ret
}

// Runs all tests under the test/testsuite directory.
fn move_check_testsuite(path: &Path) -> datatest_stable::Result<()> {
    let targets: Vec<String> = vec![path.to_str().unwrap().to_owned()];
    let deps = stdlib_files();
    let sender = Some(Address::parse_str(SENDER).unwrap());

    let exp_path = path.with_extension(EXP_EXT);
    let out_path = path.with_extension(OUT_EXT);

    let (files, units_or_errors) = move_compile_no_report(&targets, &deps, sender)?;
    let errors = match units_or_errors {
        Err(errors) => errors,
        Ok(units) => move_lang::to_bytecode::translate::verify_units(units).1,
    };
    let has_errors = !errors.is_empty();
    let error_buffer = if has_errors {
        move_lang::errors::report_errors_to_buffer(files, errors)
    } else {
        vec![]
    };

    let save_errors = read_bool_var(KEEP_TMP);
    let update_baseline = read_bool_var(UPDATE_BASELINE);

    fs::write(out_path.clone(), error_buffer)?;
    let rendered_errors = fs::read_to_string(out_path.clone())?;
    if !save_errors {
        fs::remove_file(out_path)?;
    }

    if update_baseline {
        if has_errors {
            fs::write(exp_path, rendered_errors)?;
        } else if exp_path.is_file() {
            fs::remove_file(exp_path)?;
        }
        return Ok(());
    }

    let exp_exists = exp_path.is_file();
    match (has_errors, exp_exists) {
        (false, false) => Ok(()),
        (true, false) => {
            let msg = format!("Expected success. Unexpected errors:\n{}", rendered_errors);
            error(msg)
        }
        (false, true) => {
            let msg = format!(
                "Unexpected success. Expected errors:\n{}",
                fs::read_to_string(exp_path)?
            );
            error(msg)
        }
        (true, true) => {
            let expected_errors = fs::read_to_string(exp_path)?;
            if rendered_errors != expected_errors {
                let msg = format!(
                    "Expected errors differ from actual errors:\n{}",
                    format_diff(expected_errors, rendered_errors),
                );
                error(msg)
            } else {
                Ok(())
            }
        }
    }
}

datatest_stable::harness!(move_check_testsuite, MOVE_CHECK_DIR, r".*\.move");
