// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_lang::command_line::read_bool_env_var;
use std::{fs, path::Path};

const EXP_EXT: &str = "exp";

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

// Runs all tests under the test/test_sources directory.
fn run_test(path: &Path) -> datatest_stable::Result<()> {
    std::env::set_var("NO_COLOR", "1");
    let update_baseline = read_bool_env_var(UPDATE_BASELINE) || read_bool_env_var(UPB);
    let buffer = Vec::new();
    let mut targets = move_stdlib::move_stdlib_files();
    let exp_path = path.with_extension(EXP_EXT);
    targets.push(path.to_str().unwrap().to_owned());
    let unit_test_config = move_unit_test::UnitTestingConfig {
        num_threads: 1,
        instruction_execution_bound: 1000,
        filter: None,
        source_files: targets,
        use_stackless_vm: false,
        verbose: false,
    };

    let test_plan = unit_test_config.build_test_plan();

    if test_plan.is_none() {
        return move_lang_test_utils::error(format!("No test plan constructed for {:?}", path));
    }

    let buffer = unit_test_config.run_and_report_unit_tests(test_plan.unwrap(), buffer)?;

    if update_baseline {
        fs::write(&exp_path, &buffer)?
    }

    let exp_exists = exp_path.is_file();

    if exp_exists {
        let expected = fs::read_to_string(exp_path)?;
        let output = String::from_utf8(buffer)?;
        if expected != output {
            move_lang_test_utils::error(format!(
                "Expected outputs differ:\n{}",
                format_diff(expected, output)
            ))
        } else {
            Ok(())
        }
    } else {
        move_lang_test_utils::error(format!("No expected output found for {:?}. You probably want to rerun with `env UPDATE_BASELINE=1`", path))
    }
}

datatest_stable::harness!(run_test, "tests/test_sources", r".*\.move$");
