// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

/// The on-chain consensus config, in order to be able to add fields, we use enum to wrap the actual struct.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum OnChainConsensusConfig {
    V1(ConsensusConfigV1),
}

impl OnChainConsensusConfig {
    #[allow(dead_code)]
    fn two_chain(&self) -> bool {
        match &self {
            OnChainConsensusConfig::V1(config) => config.two_chain,
        }
    }
}

/// This is used when on-chain config is not initialized.
impl Default for OnChainConsensusConfig {
    fn default() -> Self {
        OnChainConsensusConfig::V1(ConsensusConfigV1::default())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConsensusConfigV1 {
    pub two_chain: bool,
}

impl Default for ConsensusConfigV1 {
    fn default() -> Self {
        Self { two_chain: false }
    }
}

impl OnChainConfig for OnChainConsensusConfig {
    const IDENTIFIER: &'static str = "DiemConsensusConfig";

    /// The Move resource is
    /// ```ignore
    /// struct DiemConsensusConfig has copy, drop, store {
    ///    config: vector<u8>,
    /// }
    /// ```
    /// so we need two rounds of bcs deserilization to turn it back to OnChainConsensusConfig
    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_bytes: Vec<u8> = bcs::from_bytes(bytes)?;
        bcs::from_bytes(&raw_bytes)
            .map_err(|e| format_err!("[on-chain config] Failed to deserialize into config: {}", e))
    }
}
