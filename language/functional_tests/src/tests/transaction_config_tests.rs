// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{
        global::Config as GlobalConfig,
        transaction::{is_new_transaction, Config, Entry},
    },
    errors::*,
    tests::{
        global_config_tests::parse_and_build_config as parse_and_build_global_config,
        parse_each_line_as,
    },
};

#[test]
fn parse_simple_positive() {
    for s in &[
        "//! no-verify",
        "  //!no-verify ",
        "//!no-verify",
        "//! no-execute",
        "//! sender: alice",
        "//! sender:foobar42",
    ] {
        s.parse::<Entry>().unwrap();
    }
}

#[test]
fn parse_simple_negative() {
    for s in &["//!", "//! ", "//! garbage", "//! sender:"] {
        s.parse::<Entry>().unwrap_err();
    }
}

#[test]
fn parse_args() {
    for s in &[
        "//! args:",
        "//! args: 12",
        "//! args: 0xdeadbeef",
        "//! args: b\"AA\"",
        r"//! args: {{bob}}",
        "//! args: 1, 2, 3, 4",
        r"//! args: 1, 0x12, {{bob}}, {{alice}},",
    ] {
        s.parse::<Entry>().unwrap();
    }

    for s in &[
        "//!args",
        "//! args: 42xx",
        "//! args: bob",
        "//! args: \"\"",
    ] {
        s.parse::<Entry>().unwrap_err();
    }
}

#[test]
fn parse_new_transaction() {
    assert!(is_new_transaction("//! new-transaction"));
    assert!(is_new_transaction("//!new-transaction "));
    assert!(!is_new_transaction("//"));
    assert!(!is_new_transaction("//! new transaction"));
    assert!(!is_new_transaction("//! transaction"));
}

fn parse_and_build_config(global_config: &GlobalConfig, s: &str) -> Result<Config> {
    Config::build(&global_config, &parse_each_line_as::<Entry>(s)?)
}

#[rustfmt::skip]
#[test]
fn build_transaction_config_1() {
    let global = parse_and_build_global_config("").unwrap();

    parse_and_build_config(&global, r"
        //! no-verify
        //! no-execute
        //! sender: default
        //! args: 1, 2, 3
    ").unwrap();
}

#[rustfmt::skip]
#[test]
fn build_transaction_config_2() {
    let global = parse_and_build_global_config(r"
        //! account: bob
        //! account: alice
    ").unwrap();

    parse_and_build_config(&global, r"
        //! sender: alice
        //! args: {{bob}}, {{alice}}
    ").unwrap();
}

#[rustfmt::skip]
#[test]
fn build_transaction_config_3() {
    let global = parse_and_build_global_config(r"
        //! account: alice
    ").unwrap();

    parse_and_build_config(&global, r"
        //! args: {{bob}}
    ").unwrap_err();
}
