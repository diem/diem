// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_lang::command_line::read_bool_env_var;
use move_unit_test::{self, UnitTestingConfig};
use std::{
    fs,
    path::{Path, PathBuf},
};

const EXP_EXT: &str = "exp";
// We don't support statistics tests as that includes times which are variable and will make these
// tests flaky.
const TEST_MODIFIER_STRS: &[&str] = &["storage"];

const UPDATE_BASELINE: &str = "UPDATE_BASELINE";
const UPB: &str = "UPB";

fn format_diff(expected: String, actual: String) -> String {
    use difference::*;

    let changeset = Changeset::new(&expected, &actual, "\n");

    let mut ret = String::new();

    for seq in changeset.diffs {
        match &seq {
            Difference::Same(x) => {
                ret.push_str(x);
                ret.push('\n');
            }
            Difference::Add(x) => {
                ret.push_str("\x1B[92m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push('\n');
            }
            Difference::Rem(x) => {
                ret.push_str("\x1B[91m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push('\n');
            }
        }
    }
    ret
}

pub fn modify(mut base_config: UnitTestingConfig, modifier_str: &str) -> Option<UnitTestingConfig> {
    // Add future test modifiers here
    match modifier_str {
        "storage" => base_config.report_storage_on_error = true,
        _ => return None,
    };
    Some(base_config)
}

fn run_test_with_modifiers(
    unit_test_config: UnitTestingConfig,
    path: &Path,
) -> datatest_stable::Result<Vec<((Vec<u8>, bool), PathBuf)>> {
    let mut results = Vec::new();

    for modifier in TEST_MODIFIER_STRS.iter() {
        let modified_exp_path = path.with_extension(format!("{}.{}", modifier, EXP_EXT));
        if let (Some(test_config), true) = (
            modify(unit_test_config.clone(), modifier),
            modified_exp_path.exists(),
        ) {
            let buffer = Vec::new();
            let test_plan = test_config.build_test_plan();
            if test_plan.is_none() {
                move_lang_test_utils::error(format!(
                    "No test plan constructed for {:?} with modifier {}",
                    path, modifier
                ))?;
            }

            results.push((
                test_config.run_and_report_unit_tests(test_plan.unwrap(), buffer)?,
                modified_exp_path,
            ))
        }
    }

    // Now run with no modifiers
    let buffer = Vec::new();
    let test_plan = unit_test_config.build_test_plan();
    if test_plan.is_none() {
        move_lang_test_utils::error(format!("No test plan constructed for {:?}", path))?;
    }

    results.push((
        unit_test_config.run_and_report_unit_tests(test_plan.unwrap(), buffer)?,
        path.with_extension(EXP_EXT),
    ));

    Ok(results)
}

// Runs all tests under the test/test_sources directory.
fn run_test(path: &Path) -> datatest_stable::Result<()> {
    std::env::set_var("NO_COLOR", "1");
    let update_baseline = read_bool_env_var(UPDATE_BASELINE) || read_bool_env_var(UPB);
    let source_files = vec![path.to_str().unwrap().to_owned()];
    let unit_test_config = UnitTestingConfig {
        num_threads: 1,
        instruction_execution_bound: 1000,
        filter: None,
        source_files,
        dep_files: move_stdlib::move_stdlib_files(),
        check_stackless_vm: false,
        verbose: false,
        report_statistics: false,
        report_storage_on_error: false,
        list: false,
    };

    for ((buffer, _), exp_path) in run_test_with_modifiers(unit_test_config, path)? {
        if update_baseline {
            fs::write(&exp_path, &buffer)?
        }

        let exp_exists = exp_path.is_file();

        if exp_exists {
            let expected = fs::read_to_string(&exp_path)?;
            let output = String::from_utf8(buffer)?;
            if expected != output {
                return move_lang_test_utils::error(format!(
                    "Expected outputs differ for {:?}:\n{}",
                    exp_path,
                    format_diff(expected, output)
                ));
            }
        } else {
            return move_lang_test_utils::error(format!(
                "No expected output found for {:?}.\
                    You probably want to rerun with `env UPDATE_BASELINE=1`",
                path
            ));
        }
    }

    Ok(())
}

datatest_stable::harness!(run_test, "tests/test_sources", r".*\.move$");
