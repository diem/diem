// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// The config holds the options that define the testing environment.
// A config entry starts with "//!", differentiating it from a directive.

use crate::errors::*;
use std::{
    collections::{btree_map, BTreeMap},
    str::FromStr,
};
use vm_runtime_tests::account::AccountData;

// unit: microlibra
const DEFAULT_BALANCE: u64 = 1_000_000;

/// Struct that specifies the initial setup of an account.
#[derive(Debug)]
pub struct AccountDefinition {
    /// Name of the account. The name is case insensitive.
    pub name: String,
    /// The initial balance of the account.
    pub balance: Option<u64>,
    /// The initial sequence number  of the account.
    pub sequence_number: Option<u64>,
}

/// A raw entry extracted from the input. Used to build the global config table.
#[derive(Debug)]
pub enum Entry {
    /// Defines an account that can be used in tests.
    AccountDefinition(AccountDefinition),
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
            let sequence_number = match v.get(2) {
                Some(s) => Some(s.parse::<u64>()?),
                None => None,
            };
            return Ok(Entry::AccountDefinition(AccountDefinition {
                name: v[0].to_string(),
                balance,
                sequence_number,
            }));
        }
        Err(ErrorKind::Other(format!("failed to parse '{}' as global config entry", s)).into())
    }
}

/// A table of options either shared by all transactions or used to define the testing environment.
#[derive(Debug)]
pub struct Config {
    /// A map from account names to account data
    pub accounts: BTreeMap<String, AccountData>,
}

impl Config {
    pub fn build(entries: &[Entry]) -> Result<Self> {
        let mut accounts = BTreeMap::new();
        for entry in entries {
            match entry {
                Entry::AccountDefinition(def) => {
                    let account_data = AccountData::new(
                        def.balance.unwrap_or(DEFAULT_BALANCE),
                        def.sequence_number.unwrap_or(0),
                    );
                    let name = def.name.to_ascii_lowercase();
                    let entry = accounts.entry(name);
                    match entry {
                        btree_map::Entry::Vacant(entry) => {
                            entry.insert(account_data);
                        }
                        btree_map::Entry::Occupied(_) => {
                            return Err(ErrorKind::Other(format!(
                                "already has account '{}'",
                                def.name,
                            ))
                            .into());
                        }
                    }
                }
            }
        }

        if let btree_map::Entry::Vacant(entry) = accounts.entry("default".to_string()) {
            entry.insert(AccountData::new(DEFAULT_BALANCE, 0));
        }
        println!("{:?}", accounts);
        Ok(Config { accounts })
    }
}
