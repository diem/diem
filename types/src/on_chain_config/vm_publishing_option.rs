// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use diem_crypto::HashValue;
use serde::{Deserialize, Serialize};

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only allowlisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since allowlisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct VMPublishingOption {
    pub script_allow_list: Vec<HashValue>,
    pub is_open_module: bool,
}

impl VMPublishingOption {
    pub fn locked(allowlist: Vec<HashValue>) -> Self {
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

impl OnChainConfig for VMPublishingOption {
    const IDENTIFIER: &'static str = "DiemTransactionPublishingOption";
}
