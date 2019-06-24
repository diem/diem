// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// The config holds the options that define the testing environment.
// A config entry starts with "//!", differentiating it from a directive.

use crate::errors::*;
use std::{
    collections::{hash_map, HashMap},
    str::FromStr,
};
use vm_runtime_tests::account::AccountData;

const DEFAULT_BALANCE: u64 = 1_000_000;

/// A raw entry extracted from the input. Used to build the global config table.
#[derive(Debug)]
pub enum Entry {
    Account(String, Option<u64>, Option<u64>),
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s1 = s.trim_start().trim_end();
        if !s1.starts_with("//!") {
            return Err(
                ErrorKind::Other("global config entry must start with //!".to_string()).into(),
            );
        }
        let s2 = s1[3..].trim_start();
        if s2.starts_with("account:") {
            let v: Vec<_> = s2[8..]
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .collect();
            if v.is_empty() || v.len() > 3 {
                return Err(ErrorKind::Other(
                    "config 'account' takes 1 to 3 parameters".to_string(),
                )
                .into());
            }
            let balance = match v.get(1) {
                Some(s) => Some(s.parse::<u64>()?),
                None => None,
            };
            let seq_num = match v.get(2) {
                Some(s) => Some(s.parse::<u64>()?),
                None => None,
            };
            return Ok(Entry::Account(v[0].to_string(), balance, seq_num));
        }
        Err(ErrorKind::Other(format!("failed to parse '{}' as global config entry", s)).into())
    }
}

/// A table of options either shared by all transactions or used to define the testing environment.
#[derive(Debug)]
pub struct Config {
    /// A map from account names to account data
    pub accounts: HashMap<String, AccountData>,
}

impl Config {
    pub fn build(entries: &[Entry]) -> Result<Self> {
        let mut accounts = HashMap::new();

        for entry in entries {
            match entry {
                Entry::Account(name, balance, seq_number) => {
                    let account_data = AccountData::new(
                        balance.unwrap_or(DEFAULT_BALANCE),
                        seq_number.unwrap_or(0),
                    );
                    let entry = accounts.entry(name.to_ascii_lowercase());
                    match entry {
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(account_data);
                        }
                        hash_map::Entry::Occupied(_) => {
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

        if let hash_map::Entry::Vacant(entry) = accounts.entry("alice".to_string()) {
            entry.insert(AccountData::new(DEFAULT_BALANCE, 0));
        }

        Ok(Config { accounts })
    }
}
