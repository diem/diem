// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_cli::test;

use std::path::Path;

fn run_all(args_path: &Path) -> datatest_stable::Result<()> {
    test::run_one(args_path, "../../../target/debug/move", false)?;
    Ok(())
}

// runs all the tests
datatest_stable::harness!(run_all, "tests/testsuite", r"args.txt$");
