// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::global::{Config, Entry},
    errors::*,
    tests::parse_each_line_as,
};

#[test]
fn parse_account() {
    for s in &[
        "//! account: alice",
        "//!account: bob",
        "//! account: bob, 100",
        "//! account: bob, 0, 0",
    ] {
        s.parse::<Entry>().unwrap();
    }

    for s in &["//! account:", "//! account", "//! account: alice, 1, 2, 3"] {
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

    assert!(config.accounts.len() == 2);
    assert!(config.accounts.contains_key("alice"));
    let bob = config.accounts.get("bob").unwrap();
    assert!(bob.balance() == 2000);
    assert!(bob.sequence_number() == 10);
}

#[test]
fn build_global_config_2() {
    let config = parse_and_build_config("").unwrap();

    assert!(config.accounts.len() == 1);
    assert!(config.accounts.contains_key("alice"));
}

#[rustfmt::skip]
#[test]
fn build_global_config_3() {
    parse_and_build_config(r"
        //! account: bob
        //! account: BOB
    ").unwrap_err();
}
