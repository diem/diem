// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::global::{Config, Entry},
    errors::*,
    tests::parse_each_line_as,
};
use move_core_types::identifier::IdentStr;

#[test]
fn parse_account_positive() {
    for s in &[
        "//! account: alice",
        "//!account: bob",
        "//! account: bob, 100",
        "//!account:alice,",
        "//!   account :alice,1, 2",
        "//! account: bob, 0, 0",
        "//!    account : bob, 0, 0",
        "//!    account     :bob,   0,  0",
        "//!\naccount\n:bob,\n0,\n0",
        "//!\taccount\t:bob,\t0,\t0",
        "//! account: alice, 1000, 0, validator",
    ] {
        s.parse::<Entry>().unwrap();
    }
}

#[test]
fn parse_account_negative() {
    for s in &[
        "//! account:",
        "//! account",
        "//! account: alice, 1, 2, validator, 4",
    ] {
        s.parse::<Entry>().unwrap_err();
    }
}

/// Parses each line in the given input as an entry and build global config.
pub fn parse_and_build_config(s: &str) -> Result<Config> {
    Config::build(&parse_each_line_as::<Entry>(s)?)
}

#[rustfmt::skip]
#[test]
fn build_global_config_1() {
    let config = parse_and_build_config(r"
        //! account: Alice,
        //! account: bob, 2000, 10
    ").unwrap();

    assert_eq!(config.accounts.len(), 3);
    assert!(config.accounts.contains_key("default"));
    assert!(config.accounts.contains_key("alice"));
    let bob = config.accounts.get("bob").unwrap();
    assert_eq!(bob.balance(IdentStr::new("LBR").unwrap()), 2000);
    assert_eq!(bob.sequence_number(), 10);
}

#[test]
fn build_global_config_2() {
    let config = parse_and_build_config("").unwrap();

    assert_eq!(config.accounts.len(), 1);
    assert!(config.accounts.contains_key("default"));
}

#[rustfmt::skip]
#[test]
fn build_global_config_3() {
    parse_and_build_config(r"
        //! account: bob
        //! account: BOB
    ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn build_global_config_4() {
    let config = parse_and_build_config(r"
        //! account: default, 50,
    ").unwrap();

    assert_eq!(config.accounts.len(), 1);
    let default = config.accounts.get("default").unwrap();
    assert_eq!(default.balance(IdentStr::new("LBR").unwrap()), 50);
}

#[rustfmt::skip]
#[test]
fn build_global_config_5() {
    let config = parse_and_build_config(r"
        //! account: default, 50LBR,
    ").unwrap();

    assert_eq!(config.accounts.len(), 1);
    let default = config.accounts.get("default").unwrap();
    assert_eq!(default.balance(IdentStr::new("LBR").unwrap()), 50);
}

#[rustfmt::skip]
#[test]
fn build_global_config_6() {
    let config = parse_and_build_config(r"
        //! account: bob, 51Coin1,
    ").unwrap();

    assert_eq!(config.accounts.len(), 2);
    let default = config.accounts.get("bob").unwrap();
    assert_eq!(default.balance(IdentStr::new("Coin1").unwrap()), 51);
}
