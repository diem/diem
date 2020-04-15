// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use anyhow::Result;
use move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegisteredCurrencies {
    currency_codes: Vec<Identifier>,
}

impl fmt::Display for RegisteredCurrencies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for currency_code in self.currency_codes().iter() {
            write!(f, "{} ", currency_code)?;
        }
        write!(f, "]")
    }
}

impl RegisteredCurrencies {
    pub fn new(currency_codes: Vec<Identifier>) -> Self {
        Self { currency_codes }
    }

    pub fn currency_codes(&self) -> &[Identifier] {
        &self.currency_codes
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl OnChainConfig for RegisteredCurrencies {
    // registered currencies address
    const ADDRESS: &'static str = "0xA550C18";
    const IDENTIFIER: &'static str = "RegisteredCurrencies";
}
