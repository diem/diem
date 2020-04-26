// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// The config holds the options that define the testing environment.
// A config entry starts with "//!", differentiating it from a directive.

use crate::{common::strip, errors::*, genesis_accounts::make_genesis_accounts};
use language_e2e_tests::{
    account::{Account, AccountData},
    keygen::KeyGen,
};
use libra_config::generator;
use libra_crypto::PrivateKey;
use libra_types::account_config;
use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;
use std::{
    collections::{btree_map, BTreeMap},
    str::FromStr,
};

static DEFAULT_BALANCE: Lazy<Balance> = Lazy::new(|| Balance {
    amount: 1_000_000,
    currency_code: account_config::from_currency_code_string(account_config::LBR_NAME).unwrap(),
});

#[derive(Debug)]
pub enum Role {
    /// Means that the account is a current validator; its address is in the on-chain validator set
    Validator,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub amount: u64,
    pub currency_code: Identifier,
}

impl FromStr for Balance {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // TODO: Try to get this from the on-chain config?
        let coin_types = vec!["LBR", "Coin1", "Coin2"];
        let mut coin_type: Vec<&str> = coin_types.into_iter().filter(|x| s.ends_with(x)).collect();
        let currency_code = coin_type.pop().unwrap_or("LBR");
        if !coin_type.is_empty() {
            return Err(ErrorKind::Other(
                "Multiple coin types supplied for account. Accounts are single currency"
                    .to_string(),
            )
            .into());
        }
        let s = s.trim_end_matches(currency_code);
        Ok(Balance {
            amount: s.parse::<u64>()?,
            currency_code: account_config::from_currency_code_string(currency_code)?,
        })
    }
}

/// Struct that specifies the initial setup of an account.
#[derive(Debug)]
pub struct AccountDefinition {
    /// Name of the account. The name is case insensitive.
    pub name: String,
    /// The initial balance of the account.
    pub balance: Option<Balance>,
    /// The initial sequence number of the account.
    pub sequence_number: Option<u64>,
    /// Special role this account has in the system (if any)
    pub role: Option<Role>,
    /// Specifier on whether this account is an empty account or not
    pub is_empty_account_type: Option<bool>,
}

impl FromStr for Role {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "validator" => Ok(Role::Validator),
            other => Err(ErrorKind::Other(format!("Invalid account role {:?}", other)).into()),
        }
    }
}

/// A raw entry extracted from the input. Used to build the global config table.
#[derive(Debug)]
pub enum Entry {
    /// Defines an account that can be used in tests.
    AccountDefinition(AccountDefinition),
}

impl Entry {
    pub fn is_validator(&self) -> bool {
        matches!(
            self,
            Entry::AccountDefinition(AccountDefinition {
                role: Some(Role::Validator),
                ..
            })
        )
    }
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.split_whitespace().collect::<String>();
        let s = strip(&s, "//!")
            .ok_or_else(|| ErrorKind::Other("txn config entry must start with //!".to_string()))?
            .trim_start();

        if let Some(s) = strip(s, "account:") {
            let v: Vec<_> = s
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .collect();
            if v.is_empty() || v.len() > 4 {
                return Err(ErrorKind::Other(
                    "config 'account' takes 1 to 4 parameters".to_string(),
                )
                .into());
            }
            let balance = v.get(1).and_then(|s| s.parse::<Balance>().ok());
            let sequence_number = v.get(2).and_then(|s| s.parse::<u64>().ok());
            let role = v.get(3).and_then(|s| s.parse::<Role>().ok());
            // These two are mutually exclusive, so we can double-use the third position
            let is_empty_account_type = v.get(3).and_then(|s| s.parse::<bool>().ok());
            return Ok(Entry::AccountDefinition(AccountDefinition {
                name: v[0].to_string(),
                balance,
                sequence_number,
                role,
                is_empty_account_type,
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
    pub genesis_accounts: BTreeMap<String, Account>,
    /// The validator set after genesis
    pub validator_accounts: usize,
}

impl Config {
    pub fn build(entries: &[Entry]) -> Result<Self> {
        let mut accounts = BTreeMap::new();
        let mut validator_accounts = entries.iter().filter(|entry| entry.is_validator()).count();
        let total_validator_accounts = validator_accounts;

        // generate a validator set with |validator_accounts| validators
        let validator_keys = if validator_accounts > 0 {
            let mut swarm = generator::validator_swarm_for_testing(validator_accounts);
            swarm
                .nodes
                .iter_mut()
                .map(|c| {
                    let peer_id = c.validator_network.as_ref().unwrap().peer_id;
                    let account_keypair =
                        c.test.as_mut().unwrap().account_keypair.as_mut().unwrap();
                    let privkey = account_keypair.take_private().unwrap();
                    (peer_id, privkey)
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        // key generator with a fixed seed
        // this is important as it ensures the tests are deterministic
        let mut keygen = KeyGen::from_seed([0x1f; 32]);

        // initialize the keys of validator entries with the validator set
        // enhance type of config to contain a validator set, use it to initialize genesis
        for entry in entries {
            match entry {
                Entry::AccountDefinition(def) => {
                    let balance = def.balance.as_ref().unwrap_or(&DEFAULT_BALANCE).clone();
                    let account_data = if entry.is_validator() {
                        validator_accounts -= 1;
                        let privkey = &validator_keys.get(validator_accounts).unwrap().1;
                        AccountData::with_keypair(
                            privkey.clone(),
                            privkey.public_key(),
                            balance.amount,
                            balance.currency_code,
                            def.sequence_number.unwrap_or(0),
                            def.is_empty_account_type.unwrap_or(false),
                        )
                    } else {
                        let (privkey, pubkey) = keygen.generate_keypair();
                        AccountData::with_keypair(
                            privkey,
                            pubkey,
                            balance.amount,
                            balance.currency_code,
                            def.sequence_number.unwrap_or(0),
                            def.is_empty_account_type.unwrap_or(false),
                        )
                    };
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
            let (privkey, pubkey) = keygen.generate_keypair();
            entry.insert(AccountData::with_keypair(
                privkey,
                pubkey,
                DEFAULT_BALANCE.amount,
                DEFAULT_BALANCE.currency_code.clone(),
                /* sequence_number */
                0,
                /* is_empty_account_type */ false,
            ));
        }
        Ok(Config {
            accounts,
            genesis_accounts: make_genesis_accounts(),
            validator_accounts: total_validator_accounts,
        })
    }

    pub fn get_account_for_name(&self, name: &str) -> Result<&Account> {
        self.accounts
            .get(name)
            .map(|account_data| account_data.account())
            .or_else(|| self.genesis_accounts.get(name))
            .ok_or_else(|| ErrorKind::Other(format!("account '{}' does not exist", name)).into())
    }
}
