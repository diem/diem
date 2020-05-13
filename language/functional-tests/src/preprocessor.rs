// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::Directive,
    common::LineSp,
    config::{
        block_metadata::{build_block_metadata, is_new_block, Entry as BlockEntry},
        global::{Config as GlobalConfig, Entry as GlobalConfigEntry},
        transaction::{
            is_new_transaction, Config as TransactionConfig, Entry as TransactionConfigEntry,
        },
    },
    errors::*,
    evaluator::{Command, Transaction},
};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// Substitutes the placeholders (account names in double curly brackets) with addresses.
pub fn substitute_addresses(config: &GlobalConfig, text: &str) -> String {
    static PAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{([A-Za-z][A-Za-z0-9]*)\}\}").unwrap());
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

pub enum RawCommand {
    Transaction(RawTransactionInput),
    BlockMetadata(Vec<BlockEntry>),
}

fn is_empty_command(cmd: &RawCommand) -> bool {
    match cmd {
        RawCommand::Transaction(txn) => txn.text.is_empty() && txn.config_entries.is_empty(),
        RawCommand::BlockMetadata(entries) => entries.is_empty(),
    }
}

fn check_raw_transaction(txn: &RawTransactionInput) -> Result<()> {
    if txn.text.is_empty() {
        if !txn.config_entries.is_empty() {
            return Err(ErrorKind::Other(
                "config options attached to empty transaction".to_string(),
            )
            .into());
        }
        return Err(ErrorKind::Other("empty transaction".to_string()).into());
    }
    Ok(())
}

fn check_raw_command(cmd: &RawCommand) -> Result<()> {
    match cmd {
        RawCommand::Transaction(txn) => check_raw_transaction(txn),
        RawCommand::BlockMetadata(entries) => {
            if entries.len() < 2 {
                Err(
                    ErrorKind::Other("block prologue doesn't have enough arguments".to_string())
                        .into(),
                )
            } else {
                Ok(())
            }
        }
    }
}

fn new_command(input: &str) -> Option<RawCommand> {
    if is_new_transaction(input) {
        return Some(RawCommand::Transaction(RawTransactionInput {
            config_entries: vec![],
            text: vec![],
        }));
    }
    if is_new_block(input) {
        return Some(RawCommand::BlockMetadata(vec![]));
    }
    None
}

pub fn extract_global_config(
    lines: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<GlobalConfig> {
    let mut entries = vec![];

    for line in lines.into_iter() {
        let line = line.as_ref();

        if let Ok(entry) = line.parse::<GlobalConfigEntry>() {
            entries.push(entry);
            continue;
        }
    }

    GlobalConfig::build(&entries)
}

/// Parses the input string into three parts: a global config, directives and transactions.
pub fn split_input(
    lines: impl IntoIterator<Item = impl AsRef<str>>,
    global_config: &GlobalConfig,
) -> Result<(Vec<LineSp<Directive>>, Vec<RawCommand>)> {
    let mut directives = vec![];
    let mut commands = vec![];
    let mut first_transaction = true;

    let mut command = RawCommand::Transaction(RawTransactionInput {
        config_entries: vec![],
        text: vec![],
    });

    for (line_idx, line) in lines.into_iter().enumerate() {
        let line = line.as_ref();
        if let Some(new_command) = new_command(line) {
            if first_transaction && is_empty_command(&command) {
                command = new_command;
                continue;
            }
            check_raw_command(&command)?;
            commands.push(command);
            command = new_command;
            first_transaction = false;
            continue;
        }

        if let Ok(dirs) = Directive::parse_line(line) {
            directives.extend(dirs.into_iter().map(|sp| sp.into_line_sp(line_idx)));
            continue;
        }

        let line = substitute_addresses(global_config, line);
        match &mut command {
            RawCommand::Transaction(txn) => {
                if let Ok(entry) = line.parse::<TransactionConfigEntry>() {
                    txn.config_entries.push(entry);
                    continue;
                }
                // TODO: a hack to not send an empty transaction to the compiler.
                // This should go away as we refactor functional tests.
                if line.trim_start().starts_with("//") {
                    continue;
                }
                if !line.trim().is_empty() {
                    txn.text.push(line.to_string());
                    continue;
                }
            }
            RawCommand::BlockMetadata(entries) => {
                if let Ok(entry) = line.parse::<BlockEntry>() {
                    entries.push(entry);
                    continue;
                }
            }
        }
    }

    check_raw_command(&command)?;
    commands.push(command);

    Ok((directives, commands))
}

pub fn build_transactions<'a>(
    config: &'a GlobalConfig,
    command_inputs: &[RawCommand],
) -> Result<Vec<Command<'a>>> {
    command_inputs
        .iter()
        .map(|command_input| match command_input {
            RawCommand::Transaction(txn_input) => Ok(Command::Transaction(Transaction {
                config: TransactionConfig::build(config, &txn_input.config_entries)?,
                input: txn_input.text.join("\n"),
            })),
            RawCommand::BlockMetadata(entries) => Ok(Command::BlockMetadata(build_block_metadata(
                config, &entries,
            )?)),
        })
        .collect()
}
