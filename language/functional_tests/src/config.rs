// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// The config holds the options that define the testing environment.
// A config entry starts with "//!", differentiating it from a directive.

use crate::errors::*;

/// A raw config entry extracted from the input. Used to build the config.
#[derive(Debug, Clone)]
pub enum ConfigEntry {
    NoVerify,
    NoExecute,
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
}

impl Config {
    /// Builds a config from a collection of entries. Also sets the default values for entries that
    /// are missing.
    pub fn build(entries: &[ConfigEntry]) -> Result<Self> {
        let mut no_verify: Option<bool> = None;
        let mut no_execute: Option<bool> = None;
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
            }
        }
        Ok(Config {
            no_verify: no_verify.unwrap_or(false),
            no_execute: no_execute.unwrap_or(false),
        })
    }
}
