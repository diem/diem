// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use functional_tests::{
    checker::{check, Directive},
    config::{Config, ConfigEntry},
    errors::*,
    evaluator::eval,
};

fn parse_input(input: &str) -> Result<(Config, Vec<Directive>, String)> {
    let mut config_entries = vec![];
    let mut directives = vec![];
    let mut text = vec![];

    for line in input.lines() {
        if let Some(entry) = ConfigEntry::try_parse(line)? {
            config_entries.push(entry);
            continue;
        }
        if let Some(directive) = Directive::try_parse(line)? {
            directives.push(directive);
            continue;
        }
        text.push(line.to_string());
    }

    let text = text.join("\n");
    let config = Config::build(&config_entries)?;
    Ok((config, directives, text))
}

// Runs all tests under the test/testsuite directory.
#[datatest::files("tests/testsuite", { input in r".*\.mvir" })]
fn functional_tests(input: &str) -> Result<()> {
    let (config, directives, text) = parse_input(input)?;
    let res = eval(&config, &text)?;
    if let Err(e) = check(&res, &directives) {
        println!("{:#?}", res);
        return Err(e);
    }
    Ok(())
}
