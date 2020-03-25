// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::SafetyRulesConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConsensusConfig {
    pub max_block_size: u64,
    pub proposer_type: ConsensusProposerType,
    pub contiguous_rounds: u32,
    pub max_pruned_blocks_in_mem: usize,
    pub pacemaker_initial_timeout_ms: u64,
    pub safety_rules: SafetyRulesConfig,
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        ConsensusConfig {
            max_block_size: 1000,
            proposer_type: ConsensusProposerType::LeaderReputation,
            contiguous_rounds: 2,
            max_pruned_blocks_in_mem: 10000,
            pacemaker_initial_timeout_ms: 1000,
            safety_rules: SafetyRulesConfig::default(),
        }
    }
}

impl ConsensusConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.safety_rules.set_data_dir(data_dir);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusProposerType {
    // Choose the smallest PeerId as the proposer
    FixedProposer,
    // Round robin rotation of proposers
    RotatingProposer,
    // Multiple ordered proposers per round (primary, secondary, etc.)
    MultipleOrderedProposers,
    // Committed history based proposer election
    LeaderReputation,
}
