// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::Directive,
    config::{
        global::{Config as GlobalConfig, Entry as GlobalConfigEntry},
        transaction::{
            is_new_transaction, Config as TransactionConfig, Entry as TransactionConfigEntry,
        },
    },
    errors::*,
    evaluator::Transaction,
};
use language_e2e_tests::account::AccountData;
use regex::{Captures, Regex};
use std::collections::BTreeMap;

/// Substitutes the placeholders (account names in double curly brackets) with addresses.
pub fn substitute_addresses(accounts: &BTreeMap<String, AccountData>, text: &str) -> String {
    lazy_static! {
        static ref PAT: Regex = Regex::new(r"\{\{([A-Za-z][A-Za-z0-9]*)\}\}").unwrap();
    }
    PAT.replace_all(text, |caps: &Captures| {
        let name = &caps[1];
        match accounts.get(name) {
            Some(data) => format!("0x{}", data.address()),
            // TODO: find a way to return an error instead of panicking
            None => panic!(
                "account '{}' does not exist, cannot substitute address",
                name
            ),
        }
    })
    .to_string()
}

/// Parses the input string into three parts: a global config, directives and transactions.
pub fn parse_input(s: &str) -> Result<(GlobalConfig, Vec<Directive>, Vec<Transaction>)> {
    let mut global_config = vec![];
    let mut directives = vec![];
    let mut text = vec![];
    let mut transaction_config = vec![];
    let mut transactions = vec![];

    let mut first_transaction = true;

    for line in s.lines() {
        if is_new_transaction(line) {
            if text.is_empty() {
                if !transaction_config.is_empty() {
                    return Err(ErrorKind::Other(
                        "config options attached to empty transaction".to_string(),
                    )
                    .into());
                }
                if first_transaction {
                    first_transaction = false;
                    continue;
                }
                return Err(ErrorKind::Other("empty transaction".to_string()).into());
            }
            first_transaction = false;
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
        if !line.trim().is_empty() {
            text.push(line.to_string());
        }
    }

    if text.is_empty() {
        return Err(ErrorKind::Other(
            (if transaction_config.is_empty() {
                "empty transaction"
            } else {
                "config options attached to empty transaction"
            })
            .to_string(),
        )
        .into());
    }
    transactions.push((transaction_config, text));

    let global_config = GlobalConfig::build(&global_config)?;
    let transactions = transactions
        .iter()
        .map(|(config, text)| {
            let config = TransactionConfig::build(&global_config, &config)?;
            Ok(Transaction {
                config,
                input: substitute_addresses(&global_config.accounts, &text.join("\n")),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((global_config, directives, transactions))
}
