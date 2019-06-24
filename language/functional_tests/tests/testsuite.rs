// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

#[macro_use]
extern crate lazy_static;

use functional_tests::{
    checker::{check, Directive},
    config::{
        global::{Config as GlobalConfig, Entry as GlobalConfigEntry},
        transaction::{
            is_new_transaction, Config as TransactionConfig, Entry as TransactionConfigEntry,
        },
    },
    errors::*,
    evaluator::{eval, Transaction},
};
use regex::{Captures, Regex};
use std::collections::HashMap;
use vm_runtime_tests::account::AccountData;

fn substitute_addresses(accounts: &HashMap<String, AccountData>, text: &str) -> Result<String> {
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

fn parse_input(s: &str) -> Result<(GlobalConfig, Vec<Directive>, Vec<Transaction>)> {
    let mut global_config = vec![];
    let mut directives = vec![];
    let mut text = vec![];
    let mut transaction_config = vec![];
    let mut transactions = vec![];

    for line in s.lines() {
        if is_new_transaction(line) {
            // TODO: check empty
            transactions.push((transaction_config, text));
            text = vec![];
            transaction_config = vec![];
            continue;
        }
        if let Ok(entry) = line.parse::<GlobalConfigEntry>() {
            global_config.push(entry);
            continue;
        }
        if let Ok(entry) = line.parse::<TransactionConfigEntry>() {
            transaction_config.push(entry);
            continue;
        }
        if let Ok(directive) = line.parse::<Directive>() {
            directives.push(directive);
            continue;
        }
        text.push(line.to_string());
    }
    transactions.push((transaction_config, text));

    let global_config = GlobalConfig::build(&global_config)?;
    let transactions = transactions
        .iter()
        .map(|(config, text)| {
            let config = TransactionConfig::build(&global_config, &config)?;
            Ok(Transaction {
                config,
                program: substitute_addresses(&global_config.accounts, &text.join("\n"))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    println!("{:?}", transactions);
    Ok((global_config, directives, transactions))
}

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
