// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use functional_tests::{checker::check, errors::*, evaluator::eval, utils::parse_input};

// Runs all tests under the test/testsuite directory.
#[datatest::files("tests/testsuite", { input in r".*\.mvir" })]
fn functional_tests(input: &str) -> Result<()> {
    let (config, directives, transactions) = parse_input(input)?;
    let res = eval(&config, &transactions)?;
    if let Err(e) = check(&res, &directives) {
        println!("{:#?}", res);
        return Err(e);
    }
    Ok(())
}
