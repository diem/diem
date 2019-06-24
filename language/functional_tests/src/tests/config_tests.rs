// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{Config, ConfigEntry},
    errors::*,
};

#[test]
fn parse_config_entries() {
    assert!(ConfigEntry::try_parse("abc").unwrap().is_none());
    ConfigEntry::try_parse("//! no-verify").unwrap().unwrap();
    ConfigEntry::try_parse("//!no-verify   ").unwrap().unwrap();
    ConfigEntry::try_parse("//! no-execute").unwrap().unwrap();
    ConfigEntry::try_parse("//!    no-execute ")
        .unwrap()
        .unwrap();
    ConfigEntry::try_parse("//!").unwrap_err();
    ConfigEntry::try_parse("//!  ").unwrap_err();
    ConfigEntry::try_parse("//! garbage").unwrap_err();
}

fn parse_and_build_config(s: &str) -> Result<Config> {
    let mut entries = vec![];
    for line in s.lines() {
        if let Some(entry) = ConfigEntry::try_parse(line)? {
            entries.push(entry);
        }
    }
    Config::build(&entries)
}

#[rustfmt::skip]
#[test]
fn build_config() {
    parse_and_build_config(r"").unwrap();

    parse_and_build_config(r"
        //! no-verify
        //! no-execute
    ").unwrap();

    parse_and_build_config(r"
        //! no-execute
    ").unwrap();

    parse_and_build_config(r"
        //! no-verify
    ").unwrap();

    parse_and_build_config(r"
        //! no-verify
        //! no-verify
    ").unwrap_err();
}

#[test]
fn accounts_default() {
    let config = parse_and_build_config("").unwrap();
    assert!(config.accounts.len() == 1);
    assert!(config.accounts.contains_key("alice"));

    let config = parse_and_build_config("//! account: alice, 1000, 42").unwrap();
    let data = config.accounts.get("alice").unwrap();
    assert!(data.balance() == 1000);
    assert!(data.sequence_number() == 42);
}

#[rustfmt::skip]
#[test]
fn accounts_alice_bob_carol() {
    let config = parse_and_build_config(r"
        //! account: Bob, 500
        //! account: carol
    ").unwrap();
    assert!(config.accounts.len() == 3);
    assert!(config.accounts.contains_key("alice"));
    assert!(config.accounts.get("bob").unwrap().balance() == 500);
    config.accounts.get("carol").unwrap();
}

#[test]
fn accounts_ill_formed() {
    ConfigEntry::try_parse("//! account:").unwrap_err();
    ConfigEntry::try_parse("//! account: alice alice").unwrap_err();
    ConfigEntry::try_parse("//! account: bob 123 000x").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn accounts_duplicated() {
    parse_and_build_config(r"
        //! account: bob, 1000
        //! account: bob, 1000
    ").unwrap_err();
}
