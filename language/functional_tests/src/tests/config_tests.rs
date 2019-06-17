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
