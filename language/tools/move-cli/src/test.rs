// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{DEFAULT_BUILD_OUTPUT_DIR, MOVE_DATA};
use move_lang::test_utils::*;

use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
    process::Command,
};

/// Basic datatest testing framework for the CLI. The `run_one` entrypoint expects
/// an `args.txt` file with arguments that the `move` binary understands (one set
/// of arguments per line). The testing framework runs the commands, compares the
/// result to the expected output, and runs `move clean` to discard resources,
/// modules, and event data created by running the test.

const EXP_EXT: &str = "exp";

/// If this env var is set, `move clean` will not be run after each test.
/// this is useful if you want to look at the `move_data` or `move_events`
/// produced by a test. However, you'll have to manually run `move clean`
/// before re-running the test.
const NO_MOVE_CLEAN: &str = "NO_MOVE_CLEAN";
/// If either of these env vars is set, the test harness overwrites the
/// old .exp files with the output instead of checking them against the
/// output.
const UPDATE_BASELINE: &str = "UPDATE_BASELINE";
const UB: &str = "UB";

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

/// Run the `args_path` batch file with`cli_binary`
pub fn run_one(args_path: &Path, cli_binary: &str) -> anyhow::Result<()> {
    let args_file = io::BufReader::new(File::open(args_path)?).lines();
    // path where we will run the binary
    let exe_dir = args_path.parent().unwrap();
    let cli_binary_path = Path::new(cli_binary).canonicalize()?;
    let move_data = Path::new(exe_dir).join(MOVE_DATA);
    let build_output = Path::new(exe_dir).join(DEFAULT_BUILD_OUTPUT_DIR);
    assert!(
        !move_data.exists(),
        "tests should never include a {:?} directory",
        MOVE_DATA
    );
    assert!(
        !build_output.exists(),
        "tests should never include a {:?} directory",
        DEFAULT_BUILD_OUTPUT_DIR
    );
    let mut output = "".to_string();
    for args_line in args_file {
        let args_line = args_line?;
        if args_line.starts_with('#') {
            // allow comments in args.txt
            continue;
        }
        let args_iter: Vec<&str> = args_line.split_whitespace().collect();
        if args_iter.is_empty() {
            // allow blank lines in args.txt
            continue;
        }
        let cmd_output = Command::new(cli_binary_path.clone())
            .current_dir(exe_dir)
            .args(args_iter)
            .output()?;
        output += &format!("Command `{}`:\n", args_line);
        output += std::str::from_utf8(&cmd_output.stdout)?;
        output += std::str::from_utf8(&cmd_output.stderr)?;
    }

    // post-test cleanup and cleanup checks
    // check that the test command didn't create a move_src dir

    let run_move_clean = !read_bool_var(NO_MOVE_CLEAN);
    if run_move_clean {
        // run `move clean` to ensure that temporary state is cleaned up
        Command::new(cli_binary_path)
            .current_dir(exe_dir)
            .arg("clean")
            .output()?;
        // check that move_data was deleted
        assert!(
            !move_data.exists(),
            "`move clean` failed to eliminate {} directory",
            MOVE_DATA
        );
        assert!(
            !move_data.exists(),
            "`move clean` failed to eliminate {} directory",
            DEFAULT_BUILD_OUTPUT_DIR
        );
    }

    let update_baseline = read_bool_var(UPDATE_BASELINE) || read_bool_var(UB);
    let exp_path = args_path.with_extension(EXP_EXT);
    if update_baseline {
        fs::write(exp_path, &output)?;
        return Ok(());
    }
    // compare output and exp_file
    let expected_output = fs::read_to_string(exp_path).unwrap_or_else(|_| "".to_string());
    if expected_output != output {
        anyhow::bail!(
            "Expected output differs from actual output:\n{}",
            format_diff(expected_output, output)
        )
    } else {
        Ok(())
    }
}
