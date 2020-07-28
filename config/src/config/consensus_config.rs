// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::SafetyRulesConfig;
use libra_types::{account_address::AccountAddress, block_info::Round};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConsensusConfig {
    pub contiguous_rounds: u32,
    pub max_block_size: u64,
    pub max_pruned_blocks_in_mem: usize,
    pub round_initial_timeout_ms: u64,
    pub proposer_type: ConsensusProposerType,
    pub safety_rules: SafetyRulesConfig,
    // Only sync committed transactions but not vote for any pending blocks. This is useful when
    // validators coordinate on the latest version to apply a manual transaction.
    pub sync_only: bool,
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        ConsensusConfig {
            contiguous_rounds: 2,
            max_block_size: 1000,
            max_pruned_blocks_in_mem: 10000,
            round_initial_timeout_ms: 1000,
            proposer_type: ConsensusProposerType::LeaderReputation(LeaderReputationConfig {
                active_weights: 99,
                inactive_weights: 1,
            }),
            safety_rules: SafetyRulesConfig::default(),
            sync_only: false,
        }
    }
}

impl ConsensusConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.safety_rules.set_data_dir(data_dir);
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ConsensusProposerType {
    // Choose the smallest PeerId as the proposer
    FixedProposer,
    // Round robin rotation of proposers
    RotatingProposer,
    // Committed history based proposer election
    LeaderReputation(LeaderReputationConfig),
    // Pre-specified proposers for each round,
    // or default proposer if round proposer not
    // specified
    RoundProposer(HashMap<Round, AccountAddress>),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeaderReputationConfig {
    pub active_weights: u64,
    pub inactive_weights: u64,
}
