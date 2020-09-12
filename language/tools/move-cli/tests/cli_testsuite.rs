// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_cli::test;

use std::path::Path;

pub const CLI_BINARY: &str = "../../../target/debug/move-cli";
pub const CLI_TESTSUITE_DIR: &str = "tests/testsuite";

fn run_all(args_path: &Path) -> datatest_stable::Result<()> {
    Ok(test::run_one(args_path, CLI_BINARY)?)
}

datatest_stable::harness!(run_all, CLI_TESTSUITE_DIR, r"args.txt$");
