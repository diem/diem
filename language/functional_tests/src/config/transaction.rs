// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::global::Config as GlobalConfig, errors::*};
use std::str::FromStr;
use transaction_builder::transaction_argument::parse_as_transaction_argument;
use types::transaction::TransactionArgument;

/// A partially parsed transaction argument.
#[derive(Debug)]
pub enum Argument {
    AddressOf(String),
    SelfContained(TransactionArgument),
}

impl FromStr for Argument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Ok(arg) = parse_as_transaction_argument(s) {
            return Ok(Argument::SelfContained(arg));
        }
        if s.starts_with("{{") && s.ends_with("}}") {
            return Ok(Argument::AddressOf(s[2..s.len() - 2].to_string()));
        }
        Err(ErrorKind::Other(format!("failed to parse '{}' as argument", s)).into())
    }
}

/// A raw entry extracted from the input. Used to build a transaction config table.
#[derive(Debug)]
pub enum Entry {
    NoVerify,
    NoExecute,
    Sender(String),
    Arguments(Vec<Argument>),
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim_start().trim_end();
        if !s.starts_with("//!") {
            return Err(
                ErrorKind::Other("txn config entry must start with //!".to_string()).into(),
            );
        }
        let s = s[3..].trim_start();
        match s {
            "no-verify" => {
                return Ok(Entry::NoVerify);
            }
            "no-execute" => {
                return Ok(Entry::NoExecute);
            }
            _ => {}
        }
        if s.starts_with("sender:") {
            let s = s[7..].trim_start().trim_end();
            if s.is_empty() {
                return Err(ErrorKind::Other("sender cannot be empty".to_string()).into());
            }
            return Ok(Entry::Sender(s.to_string()));
        }
        if s.starts_with("args:") {
            let res: Result<Vec<_>> = s[5..]
                .split(',')
                .map(|s| s.trim_start().trim_end())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Argument>())
                .collect();
            return Ok(Entry::Arguments(res?));
        }
        Err(ErrorKind::Other(format!(
            "failed to parse '{}' as transaction config entry",
            s
        ))
        .into())
    }
}

/// Checks whether a line denotes the start of a new transaction.
pub fn is_new_transaction(s: &str) -> bool {
    if !s.starts_with("//!") {
        return false;
    }
    s[3..].trim_start().trim_end() == "new-transaction"
}

impl Entry {
    pub fn try_parse(s: &str) -> Result<Option<Self>> {
        if s.starts_with("//!") {
            Ok(Some(s.parse::<Entry>()?))
        } else {
            Ok(None)
        }
    }
}

/// A table of options specific to one transaction, fine tweaking how the transaction
/// is handled by the testing infra.
#[derive(Debug)]
pub struct Config {
    pub no_verify: bool,
    pub no_execute: bool,
    pub sender: String,
    pub args: Vec<TransactionArgument>,
}

impl Config {
    /// Builds a transaction config table from raw entries.
    pub fn build(config: &GlobalConfig, entries: &[Entry]) -> Result<Self> {
        let mut no_verify = None;
        let mut no_execute = None;
        let mut sender = None;
        let mut args = None;

        for entry in entries {
            match entry {
                Entry::NoVerify => match no_verify {
                    None => {
                        no_verify = Some(true);
                    }
                    _ => {
                        return Err(
                            ErrorKind::Other("flag 'no-verify' already set".to_string()).into()
                        );
                    }
                },
                Entry::NoExecute => match no_execute {
                    None => {
                        no_execute = Some(true);
                    }
                    _ => {
                        return Err(
                            ErrorKind::Other("flag 'no-execute' already set".to_string()).into(),
                        );
                    }
                },
                Entry::Sender(name) => match sender {
                    None => {
                        if config.accounts.contains_key(name) {
                            sender = Some(name.to_string())
                        } else {
                            return Err(ErrorKind::Other(format!(
                                "account '{}' does not exist",
                                name
                            ))
                            .into());
                        }
                    }
                    _ => return Err(ErrorKind::Other("sender already set".to_string()).into()),
                },
                Entry::Arguments(raw_args) => match args {
                    None => {
                        args = Some(
                            raw_args
                                .iter()
                                .map(|arg| match arg {
                                    Argument::AddressOf(name) => match config.accounts.get(name) {
                                        Some(data) => {
                                            Ok(TransactionArgument::Address(*data.address()))
                                        }
                                        None => Err(ErrorKind::Other(format!(
                                            "account '{}' does not exist",
                                            name
                                        ))
                                        .into()),
                                    },
                                    Argument::SelfContained(arg) => Ok(arg.clone()),
                                })
                                .collect::<Result<Vec<_>>>()?,
                        );
                    }
                    _ => {
                        return Err(ErrorKind::Other(
                            "transaction arguments already set".to_string(),
                        )
                        .into())
                    }
                },
            }
        }

        Ok(Config {
            no_verify: no_verify.unwrap_or(false),
            no_execute: no_execute.unwrap_or(false),
            sender: sender.unwrap_or_else(|| "alice".to_string()),
            args: args.unwrap_or_else(|| vec![]),
        })
    }
}
