// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{
        block_metadata::{build_block_metadata, is_new_block, Entry},
        global::Config as GlobalConfig,
    },
    errors::*,
    tests::{
        global_config_tests::parse_and_build_config as parse_and_build_global_config,
        parse_each_line_as,
    },
};
use libra_types::block_metadata::BlockMetadata;

#[test]
fn parse_simple_positive() {
    for s in &[
        "//! proposer: alice",
        "//! proposer\t:\tfoobar42",
        "//!\nproposer\n:\nfoobar42",
    ] {
        s.parse::<Entry>().unwrap();
    }
}

#[test]
fn parse_simple_negative() {
    for s in &["//!", "//! ", "//! sender: alice", "//! proposer:"] {
        s.parse::<Entry>().unwrap_err();
    }
}

#[test]
fn parse_timestamp() {
    for s in &[
        "//! block-time:77",
        "//!block-time:0",
        "//! block-time:  123",
    ] {
        s.parse::<Entry>().unwrap();
    }

    for s in &[
        "//!block-time:",
        "//!block-time:abc",
        "//!block-time: 123, 45",
    ] {
        s.parse::<Entry>().unwrap_err();
    }
}

#[test]
fn parse_new_transaction() {
    assert!(is_new_block("//! block-prologue"));
    assert!(is_new_block("//!block-prologue "));
    assert!(!is_new_block("//"));
    assert!(!is_new_block("//! new block"));
    assert!(!is_new_block("//! block"));
}

fn parse_and_build_config(global_config: &GlobalConfig, s: &str) -> Result<BlockMetadata> {
    build_block_metadata(&global_config, &parse_each_line_as::<Entry>(s)?)
}

#[rustfmt::skip]
#[test]
fn build_transaction_config_1() {
    let global = parse_and_build_global_config(r"
        //! account: alice
    ").unwrap();

    parse_and_build_config(&global, r"
        //! proposer: alice
        //! block-time: 6
    ").unwrap();

    parse_and_build_config(&global, r"
        //! proposer: alice
    ").unwrap_err();

    parse_and_build_config(&global, r"
        //! block-time: 6
    ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn build_transaction_config_3() {
    let global = parse_and_build_global_config(r"
        //! account: alice
    ").unwrap();

    parse_and_build_config(&global, r"
        //! proposer: bob
        //! block-time: 6
    ").unwrap_err();
}
