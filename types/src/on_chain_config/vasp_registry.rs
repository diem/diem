// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VASPRegistry {
    registered_vasps: Vec<AccountAddress>,
}

impl fmt::Display for VASPRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for vasp_address in self.registered_vasps.iter() {
            write!(f, "{} ", vasp_address)?;
        }
        write!(f, "]")
    }
}

impl VASPRegistry {
    pub fn new(registered_vasps: Vec<AccountAddress>) -> Self {
        Self { registered_vasps }
    }

    pub fn registered_vasps(&self) -> &[AccountAddress] {
        &self.registered_vasps
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl OnChainConfig for VASPRegistry {
    const IDENTIFIER: &'static str = "VASPRegistry";
}
