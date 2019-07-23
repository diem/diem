// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::{self, format_err};
use std::convert::TryFrom;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// Libra protocols should not be confused by other instances of the Libra
/// protocol that may be running concurrently. For instance, discovery notes made
/// on Libra testnet should not be valid on Libra mainnet. A ChainId allows us to
/// provide domain separation between Libra instances.
///
/// Current mappings:
///
/// Libra mainnet <=> 1
/// Libra testnet <=> 10
pub struct ChainId(u32);

impl ChainId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// The chain id for Libra mainnet.
    pub const fn mainnet() -> Self {
        Self(1)
    }

    /// The chain id for Libra testnet.
    pub const fn testnet() -> Self {
        Self(10)
    }

    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl TryFrom<&str> for ChainId {
    type Error = failure::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "mainnet" | "libra-mainnet" => Ok(Self::mainnet()),
            "testnet" | "libra-testnet" => Ok(Self::testnet()),
            s => Err(format_err!("Unknown chain id string: \"{}\"", s)),
        }
    }
}
