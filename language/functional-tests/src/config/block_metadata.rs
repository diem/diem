// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::strip, config::global::Config as GlobalConfig, errors::*};
use libra_crypto::HashValue;
use libra_types::block_metadata::BlockMetadata;
use std::{collections::btree_map::BTreeMap, str::FromStr};

#[derive(Debug)]
pub enum Entry {
    Proposer(String),
    Timestamp(u64),
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.split_whitespace().collect::<String>();
        let s = strip(&s, "//!")
            .ok_or_else(|| ErrorKind::Other("txn config entry must start with //!".to_string()))?
            .trim_start();

        if let Some(s) = strip(s, "proposer:") {
            if s.is_empty() {
                return Err(ErrorKind::Other("sender cannot be empty".to_string()).into());
            }
            return Ok(Entry::Proposer(s.to_string()));
        }

        if let Some(s) = strip(s, "block-time:") {
            return Ok(Entry::Timestamp(s.parse::<u64>()?));
        }
        Err(ErrorKind::Other(format!(
            "failed to parse '{}' as transaction config entry",
            s
        ))
        .into())
    }
}

/// Checks whether a line denotes the start of a new transaction.
pub fn is_new_block(s: &str) -> bool {
    let s = s.trim();
    if !s.starts_with("//!") {
        return false;
    }
    s[3..].trim_start() == "block-prologue"
}

impl Entry {
    pub fn try_parse(s: &str) -> Result<Option<Self>> {
        if s.starts_with("//!") {
            Ok(Some(s.parse::<Entry>()?))
        } else {
            Ok(None)
        }
    }
}

pub fn build_block_metadata(config: &GlobalConfig, entries: &[Entry]) -> Result<BlockMetadata> {
    let mut timestamp = None;
    let mut proposer = None;
    for entry in entries {
        match entry {
            Entry::Proposer(s) => {
                proposer = Some(*config.get_account_for_name(s)?.address());
            }
            Entry::Timestamp(new_timestamp) => timestamp = Some(new_timestamp),
        }
    }
    if let (Some(t), Some(addr)) = (timestamp, proposer) {
        // TODO: Add parser for hash value and vote maps.
        Ok(BlockMetadata::new(
            HashValue::zero(),
            0,
            *t,
            BTreeMap::new(),
            addr,
        ))
    } else {
        Err(ErrorKind::Other("Cannot generate block metadata".to_string()).into())
    }
}
