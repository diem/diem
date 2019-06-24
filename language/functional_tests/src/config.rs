// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// The config holds the options that define the testing environment.
// A config entry starts with "//!", differentiating it from a directive.

use crate::errors::*;
use std::collections::{btree_map, BTreeMap};
use vm_runtime_tests::account::AccountData;

/// A raw config entry extracted from the input. Used to build the config.
#[derive(Debug, Clone)]
pub enum ConfigEntry {
    NoVerify,
    NoExecute,
    Account(String, u64, Option<u64>),
}

impl ConfigEntry {
    /// Tries to parse the input as an entry. Errors when the input looks
    /// like a config but is ill-formed.
    pub fn try_parse(s: &str) -> Result<Option<Self>> {
        let s1 = s.trim_start().trim_end();
        if !s1.starts_with("//!") {
            return Ok(None);
        }
        let s2 = s1[3..].trim_start();
        match s2 {
            "no-verify" => {
                return Ok(Some(ConfigEntry::NoVerify));
            }
            "no-execute" => {
                return Ok(Some(ConfigEntry::NoExecute));
            }
            _ => {}
        }
        if s2.starts_with("account:") {
            let v: Vec<_> = s2[8..]
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .collect();
            if v.len() < 2 || v.len() > 3 {
                return Err(ErrorKind::Other(
                    "config 'account' takes either 2 or 3 parameters".to_string(),
                )
                .into());
            }
            return Ok(Some(ConfigEntry::Account(
                v[0].to_ascii_lowercase(),
                v[1].parse::<u64>()?,
                if v.len() > 2 {
                    Some(v[2].parse::<u64>()?)
                } else {
                    None
                },
            )));
        }
        Err(ErrorKind::Other(format!("invalid config option '{:?}'", s2)).into())
    }
}

/// A table of options that customizes/defines the testing environment.
#[derive(Debug)]
pub struct Config {
    /// If set to true, the compiled program is sent through execution without being verified
    pub no_verify: bool,
    /// If set to true, the compiled program will not get executed
    pub no_execute: bool,
    /// A map from account names to account data
    pub accounts: BTreeMap<String, AccountData>,
}

impl Config {
    /// Builds a config from a collection of entries. Also sets the default values for entries that
    /// are missing.
    pub fn build(entries: &[ConfigEntry]) -> Result<Self> {
        let mut no_verify = None;
        let mut no_execute = None;
        let mut accounts: BTreeMap<String, _> = BTreeMap::new();
        for entry in entries {
            match entry {
                ConfigEntry::NoVerify => match no_verify {
                    None => {
                        no_verify = Some(true);
                    }
                    _ => {
                        return Err(
                            ErrorKind::Other("flag 'no-verify' already set".to_string()).into()
                        );
                    }
                },
                ConfigEntry::NoExecute => match no_execute {
                    None => {
                        no_execute = Some(true);
                    }
                    _ => {
                        return Err(
                            ErrorKind::Other("flag 'no-execute' already set".to_string()).into(),
                        );
                    }
                },
                ConfigEntry::Account(name, balance, seq_number) => {
                    let account_data = AccountData::new(*balance, seq_number.unwrap_or(0));
                    let entry = accounts.entry(name.to_string());
                    match entry {
                        btree_map::Entry::Vacant(entry) => {
                            entry.insert(account_data);
                        }
                        btree_map::Entry::Occupied(_) => {
                            return Err(ErrorKind::Other(format!(
                                "already has account '{}'",
                                name
                            ))
                            .into());
                        }
                    }
                }
            }
        }
        if let btree_map::Entry::Vacant(entry) = accounts.entry("alice".to_string()) {
            entry.insert(AccountData::new(1_000_000, 0));
        }
        Ok(Config {
            no_verify: no_verify.unwrap_or(false),
            no_execute: no_execute.unwrap_or(false),
            accounts,
        })
    }
}
