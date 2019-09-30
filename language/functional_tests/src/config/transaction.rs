// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::global::Config as GlobalConfig, errors::*, evaluator::Stage};
use std::{collections::BTreeSet, str::FromStr};
use types::transaction::{parse_as_transaction_argument, TransactionArgument};

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
    DisableStages(Vec<Stage>),
    Sender(String),
    Arguments(Vec<Argument>),
    Receiver(String),
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.split_whitespace().collect::<String>();
        if !s.starts_with("//!") {
            return Err(
                ErrorKind::Other("txn config entry must start with //!".to_string()).into(),
            );
        }
        let s = s[3..].trim_start();
        if s.starts_with("sender:") {
            let s = s[7..].trim_start().trim_end();
            if s.is_empty() {
                return Err(ErrorKind::Other("sender cannot be empty".to_string()).into());
            }
            return Ok(Entry::Sender(s.to_ascii_lowercase()));
        }
        if s.starts_with("receiver:") {
            let s = s[9..].trim_start().trim_end();
            if s.is_empty() {
                return Err(ErrorKind::Other("receiver cannot be empty".to_string()).into());
            }
            return Ok(Entry::Receiver(s.to_ascii_lowercase()));
        }
        if s.starts_with("args:") {
            let res: Result<Vec<_>> = s[5..]
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Argument>())
                .collect();
            return Ok(Entry::Arguments(res?));
        }
        if s.starts_with("no-run:") {
            let res: Result<Vec<_>> = s[7..]
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Stage>())
                .collect();
            return Ok(Entry::DisableStages(res?));
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
    let s = s.trim();
    if !s.starts_with("//!") {
        return false;
    }
    s[3..].trim_start() == "new-transaction"
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
    pub disabled_stages: BTreeSet<Stage>,
    pub sender: String,
    pub args: Vec<TransactionArgument>,
    pub receiver: Option<String>,
}

impl Config {
    /// Builds a transaction config table from raw entries.
    pub fn build(config: &GlobalConfig, entries: &[Entry]) -> Result<Self> {
        let mut disabled_stages = BTreeSet::new();
        let mut sender = None;
        let mut args = None;
        let mut receiver = None;

        for entry in entries {
            match entry {
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
                Entry::Receiver(name) => match receiver {
                    None => {
                        if config.accounts.contains_key(name) {
                            receiver = Some(name.to_string())
                        } else {
                            return Err(ErrorKind::Other(format!(
                                "account '{}' does not exist",
                                name
                            ))
                            .into());
                        }
                    }
                    _ => return Err(ErrorKind::Other("receiver already set".to_string()).into()),
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
                Entry::DisableStages(stages) => {
                    for stage in stages {
                        if !disabled_stages.insert(*stage) {
                            return Err(ErrorKind::Other(format!(
                                "duplicate stage '{:?}' in black list",
                                stage
                            ))
                            .into());
                        }
                    }
                }
            }
        }

        Ok(Config {
            disabled_stages,
            sender: sender.unwrap_or_else(|| "default".to_string()),
            args: args.unwrap_or_else(|| vec![]),
            receiver,
        })
    }

    #[inline]
    pub fn is_stage_disabled(&self, stage: Stage) -> bool {
        self.disabled_stages.contains(&stage)
    }
}
