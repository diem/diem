// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::Directive,
    common::LineSp,
    config::{
        global::{Config as GlobalConfig, Entry as GlobalConfigEntry},
        transaction::{
            is_new_transaction, Config as TransactionConfig, Entry as TransactionConfigEntry,
        },
    },
    errors::*,
    evaluator::Transaction,
};
use regex::{Captures, Regex};

/// Substitutes the placeholders (account names in double curly brackets) with addresses.
pub fn substitute_addresses(config: &GlobalConfig, text: &str) -> String {
    lazy_static! {
        static ref PAT: Regex = Regex::new(r"\{\{([A-Za-z][A-Za-z0-9]*)\}\}").unwrap();
    }
    PAT.replace_all(text, |caps: &Captures| {
        let name = &caps[1];

        format!("0x{}", config.get_account_for_name(name).unwrap().address())
    })
    .to_string()
}

pub struct RawTransactionInput {
    pub config_entries: Vec<TransactionConfigEntry>,
    pub text: Vec<String>,
}

/// Parses the input string into three parts: a global config, directives and transactions.
pub fn split_input(
    lines: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<(
    Vec<GlobalConfigEntry>,
    Vec<LineSp<Directive>>,
    Vec<RawTransactionInput>,
)> {
    let mut global_config = vec![];
    let mut directives = vec![];
    let mut text = vec![];
    let mut transaction_config = vec![];
    let mut transactions = vec![];

    let mut first_transaction = true;

    for (line_idx, line) in lines.into_iter().enumerate() {
        let line = line.as_ref();
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
            transactions.push(RawTransactionInput {
                config_entries: transaction_config,
                text,
            });
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
        if let Ok(dirs) = Directive::parse_line(line) {
            directives.extend(dirs.into_iter().map(|sp| sp.into_line_sp(line_idx)));
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
    transactions.push(RawTransactionInput {
        config_entries: transaction_config,
        text,
    });

    Ok((global_config, directives, transactions))
}

pub fn build_transactions<'a>(
    config: &'a GlobalConfig,
    txn_inputs: &[RawTransactionInput],
) -> Result<Vec<Transaction<'a>>> {
    txn_inputs
        .iter()
        .map(|txn_input| {
            Ok(Transaction {
                config: TransactionConfig::build(config, &txn_input.config_entries)?,
                input: substitute_addresses(config, &txn_input.text.join("\n")),
            })
        })
        .collect()
}
