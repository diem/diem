// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{on_chain_config::OnChainConfig, transaction::SCRIPT_HASH_LENGTH};
use anyhow::{format_err, Result};
use libra_crypto::HashValue;
use serde::{Deserialize, Serialize};

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum VMPublishingOption {
    /// Only allow scripts on a whitelist to be run
    Locked(Vec<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow custom scripts, but _not_ custom module publishing
    CustomScripts,
    /// Allow both custom scripts and custom module publishing
    Open,
}

impl OnChainConfig for VMPublishingOption {
    const IDENTIFIER: &'static str = "ScriptWhitelist";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes::<Vec<u8>>(&bytes)
            .map_err(|e| {
                format_err!(
                    "Failed first round of deserialization for VMPublishingOption: {}",
                    e
                )
            })
            .and_then(|bytes| Self::deserialize_default_impl(&bytes))
    }
}

impl VMPublishingOption {
    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn is_allowed_script(&self, program: &[u8]) -> bool {
        match self {
            VMPublishingOption::Open | VMPublishingOption::CustomScripts => true,
            VMPublishingOption::Locked(whitelist) => {
                let hash_value = HashValue::from_sha3_256(program);
                whitelist.contains(hash_value.as_ref())
            }
        }
    }
}
