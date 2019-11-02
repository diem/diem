// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use functional_tests::{
    checker::check,
    config::global::Config as GlobalConfig,
    evaluator::eval,
    utils::{build_transactions, split_input},
};
use std::{fs::read_to_string, path::Path};

// Runs all tests under the test/testsuite directory.
fn functional_tests(path: &Path) -> datatest_stable::Result<()> {
    let input = read_to_string(path)?;

    let (config, directives, transactions) = split_input(&input)?;
    let config = GlobalConfig::build(&config)?;
    let transactions = build_transactions(&config, &transactions)?;

    let log = eval(&config, &transactions)?;
    if let Err(e) = check(&log, &directives) {
        // TODO: allow the user to select debug/display mode
        println!("{}", log);
        return Err(e.into());
    }
    Ok(())
}

datatest_stable::harness!(functional_tests, "tests/testsuite", r".*\.mvir");
