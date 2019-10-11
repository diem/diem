// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use functional_tests::{checker::check, evaluator::eval, utils::parse_input};
use std::{fs::File, io::Read, path::Path};

// Runs all tests under the test/testsuite directory.
fn functional_tests(path: &Path) -> datatest_stable::Result<()> {
    let mut file = File::open(path)?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;

    let (config, directives, transactions) = parse_input(&input)?;
    let log = eval(&config, &transactions)?;
    if let Err(e) = check(&log, &directives) {
        // TODO: allow the user to select debug/display mode
        println!("{}", log);
        return Err(e.into());
    }
    Ok(())
}

datatest_stable::harness!(functional_tests, "tests/testsuite", r".*\.mvir");
