// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{on_chain_config::OnChainConfig, transaction::SCRIPT_HASH_LENGTH};
use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only allowlisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since allowlisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMPublishingOption {
    pub script_allow_list: Vec<[u8; SCRIPT_HASH_LENGTH]>,
    pub is_open_module: bool,
}

impl VMPublishingOption {
    pub fn locked(allowlist: Vec<[u8; SCRIPT_HASH_LENGTH]>) -> Self {
        Self {
            script_allow_list: allowlist,
            is_open_module: false,
        }
    }

    pub fn custom_scripts() -> Self {
        Self {
            script_allow_list: vec![],
            is_open_module: false,
        }
    }

    pub fn open() -> Self {
        Self {
            script_allow_list: vec![],
            is_open_module: true,
        }
    }

    pub fn is_open_module(&self) -> bool {
        self.is_open_module
    }

    pub fn is_open_script(&self) -> bool {
        self.script_allow_list.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct VMPublishingOptionInner {
    pub script_allow_list: Vec<Vec<u8>>,
    pub is_open_module: bool,
}

impl OnChainConfig for VMPublishingOption {
    const IDENTIFIER: &'static str = "DiemTransactionPublishingOption";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_publishing_option =
            bcs::from_bytes::<VMPublishingOptionInner>(&bytes).map_err(|e| {
                format_err!(
                    "Failed first round of deserialization for VMPublishingOptionInner: {}",
                    e
                )
            })?;
        Ok(VMPublishingOption {
            script_allow_list: raw_publishing_option
                .script_allow_list
                .into_iter()
                .map(|v| {
                    let mut hash: [u8; SCRIPT_HASH_LENGTH] = [0; SCRIPT_HASH_LENGTH];
                    hash.copy_from_slice(v.as_ref());
                    hash
                })
                .collect(),
            is_open_module: raw_publishing_option.is_open_module,
        })
    }
}
