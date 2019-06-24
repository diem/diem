// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::global::Config as GlobalConfig, errors::*};
use hex;
use std::{convert::TryFrom, str::FromStr};
use types::{
    account_address::AccountAddress, byte_array::ByteArray, transaction::TransactionArgument,
};

/// A partially parsed transaction argument.
#[derive(Debug)]
pub enum Argument {
    AddressOf(String),
    SelfContained(TransactionArgument),
}

impl Argument {
    /// Parses the string as address of some account defined in the global config.
    pub fn parse_as_address_of(s: &str) -> Result<Self> {
        if s.starts_with("{{") && s.ends_with("}}") {
            return Ok(Argument::AddressOf(s[2..s.len() - 2].to_string()));
        }
        Err(ErrorKind::Other("not address of".to_string()).into())
    }

    /// Parses the string as address.
    pub fn parse_as_address(s: &str) -> Result<Self> {
        if !s.starts_with("0x") && !s.starts_with("0X") {
            return Err(ErrorKind::Other("not address".to_string()).into());
        }
        let mut addr = hex::decode(&s[2..])?;
        if addr.len() > 32 {
            return Err(ErrorKind::Other("address must be less than 32 bytes".to_string()).into());
        }
        if addr.len() < 32 {
            addr = [0 as u8]
                .repeat(32 - addr.len())
                .into_iter()
                .chain(addr.into_iter())
                .collect();
        }
        Ok(Argument::SelfContained(TransactionArgument::Address(
            AccountAddress::try_from(addr)?,
        )))
    }

    /// Parses the string as bytearray.
    pub fn parse_as_byte_array(s: &str) -> Result<Self> {
        if s.starts_with("b\"") && s.ends_with('"') && s.len() >= 3 {
            Ok(Argument::SelfContained(TransactionArgument::ByteArray(
                ByteArray::new(hex::decode(&s[2..s.len() - 1])?),
            )))
        } else {
            Err(ErrorKind::Other("not byte array".to_string()).into())
        }
    }

    /// Parses the string as u64.
    pub fn parse_as_u64(s: &str) -> Result<Self> {
        let x = s.parse::<u64>()?;
        Ok(Argument::SelfContained(TransactionArgument::U64(x)))
    }
}

macro_rules! return_if_ok {
    ($e: expr) => {{
        if let Ok(res) = $e {
            return Ok(res);
        }
    }};
}

impl FromStr for Argument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        return_if_ok!(Argument::parse_as_address_of(s));
        return_if_ok!(Argument::parse_as_address(s));
        return_if_ok!(Argument::parse_as_byte_array(s));
        return_if_ok!(Argument::parse_as_u64(s));
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
