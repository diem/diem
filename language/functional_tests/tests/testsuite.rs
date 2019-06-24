// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

#[macro_use]
extern crate lazy_static;

use functional_tests::{
    checker::{check, Directive},
    config::{Config, ConfigEntry},
    errors::*,
    evaluator::eval,
};
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use vm_runtime_tests::account::AccountData;

fn substitute_addresses(accounts: &BTreeMap<String, AccountData>, text: &str) -> Result<String> {
    lazy_static! {
        static ref PAT: Regex = Regex::new(r"\{\{([A-Za-z][A-Za-z0-9]*)\}\}").unwrap();
    }
    Ok(PAT
        .replace_all(text, |caps: &Captures| {
            let name = &caps[1];
            match accounts.get(name) {
                Some(data) => format!("0x{}", data.address()),
                None => panic!(
                    "account '{}' does not exist, cannot substitute address",
                    name
                ),
            }
        })
        .to_string())
}

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
    let text = substitute_addresses(&config.accounts, &text)?;
    let res = eval(&config, &text)?;
    if let Err(e) = check(&res, &directives) {
        println!("{:#?}", res);
        return Err(e);
    }
    Ok(())
}
