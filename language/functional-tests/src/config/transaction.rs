// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::strip,
    config::{global::Config as GlobalConfig, type_tag_parser::parse_type_tags},
    errors::*,
    evaluator::Stage,
};
use language_e2e_tests::account::Account;
use libra_types::transaction::{parse_as_transaction_argument, TransactionArgument};
use move_core_types::language_storage::TypeTag;
use std::{collections::BTreeSet, str::FromStr, time::Duration};

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
    TypeArguments(Vec<TypeTag>),
    Arguments(Vec<Argument>),
    MaxGas(u64),
    GasPrice(u64),
    SequenceNumber(u64),
    ExpirationTime(u64),
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.split_whitespace().collect::<String>();
        let s = strip(&s, "//!")
            .ok_or_else(|| ErrorKind::Other("txn config entry must start with //!".to_string()))?
            .trim_start();

        if let Some(s) = strip(s, "sender:") {
            if s.is_empty() {
                return Err(ErrorKind::Other("sender cannot be empty".to_string()).into());
            }
            return Ok(Entry::Sender(s.to_ascii_lowercase()));
        }
        if let Some(s) = strip(s, "type-args:") {
            return Ok(Entry::TypeArguments(parse_type_tags(s)?));
        }
        if let Some(s) = strip(s, "args:") {
            let res: Result<Vec<_>> = s
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Argument>())
                .collect();
            return Ok(Entry::Arguments(res?));
        }
        if let Some(s) = strip(s, "no-run:") {
            let res: Result<Vec<_>> = s
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Stage>())
                .collect();
            return Ok(Entry::DisableStages(res?));
        }
        if let Some(s) = strip(s, "max-gas:") {
            return Ok(Entry::MaxGas(s.parse::<u64>()?));
        }
        if let Some(s) = strip(s, "gas-price:") {
            return Ok(Entry::GasPrice(s.parse::<u64>()?));
        }
        if let Some(s) = strip(s, "sequence-number:") {
            return Ok(Entry::SequenceNumber(s.parse::<u64>()?));
        }
        if let Some(s) = strip(s, "expiration-time:") {
            return Ok(Entry::ExpirationTime(s.parse::<u64>()?));
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
pub struct Config<'a> {
    pub disabled_stages: BTreeSet<Stage>,
    pub sender: &'a Account,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<TransactionArgument>,
    pub max_gas: Option<u64>,
    pub gas_price: Option<u64>,
    pub sequence_number: Option<u64>,
    pub expiration_time: Option<Duration>,
}

impl<'a> Config<'a> {
    /// Builds a transaction config table from raw entries.
    pub fn build(config: &'a GlobalConfig, entries: &[Entry]) -> Result<Self> {
        let mut disabled_stages = BTreeSet::new();
        let mut sender = None;
        let mut ty_args = None;
        let mut args = None;
        let mut max_gas = None;
        let mut gas_price = None;
        let mut sequence_number = None;
        let mut expiration_time = None;

        for entry in entries {
            match entry {
                Entry::Sender(name) => match sender {
                    None => sender = Some(config.get_account_for_name(name)?),
                    _ => return Err(ErrorKind::Other("sender already set".to_string()).into()),
                },
                Entry::TypeArguments(ty_args_) => match ty_args {
                    None => {
                        ty_args = Some(ty_args_.clone());
                    }
                    _ => bail!("transaction type arguments already set"),
                },
                Entry::Arguments(raw_args) => match args {
                    None => {
                        args = Some(
                            raw_args
                                .iter()
                                .map(|arg| match arg {
                                    Argument::AddressOf(name) => Ok(TransactionArgument::Address(
                                        *config.get_account_for_name(name)?.address(),
                                    )),
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
                Entry::MaxGas(n) => match max_gas {
                    None => max_gas = Some(*n),
                    Some(_) => {
                        return Err(
                            ErrorKind::Other("max gas amount already set".to_string()).into()
                        )
                    }
                },
                Entry::GasPrice(n) => match gas_price {
                    None => gas_price = Some(*n),
                    Some(_) => {
                        return Err(ErrorKind::Other("gas price already set".to_string()).into())
                    }
                },
                Entry::SequenceNumber(sn) => match sequence_number {
                    None => sequence_number = Some(*sn),
                    Some(_) => {
                        return Err(
                            ErrorKind::Other("sequence number already set".to_string()).into()
                        )
                    }
                },
                Entry::ExpirationTime(sn) => match expiration_time {
                    None => expiration_time = Some(Duration::from_secs(*sn)),
                    Some(_) => {
                        return Err(
                            ErrorKind::Other("expiration time already set".to_string()).into()
                        )
                    }
                },
            }
        }

        Ok(Self {
            disabled_stages,
            sender: sender.unwrap_or_else(|| config.accounts.get("default").unwrap().account()),
            ty_args: ty_args.unwrap_or_else(|| vec![]),
            args: args.unwrap_or_else(|| vec![]),
            max_gas,
            gas_price,
            sequence_number,
            expiration_time,
        })
    }

    #[inline]
    pub fn is_stage_disabled(&self, stage: Stage) -> bool {
        self.disabled_stages.contains(&stage)
    }
}
