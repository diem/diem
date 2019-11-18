// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{PersistableConfig, SafetyRulesBackend, SafetyRulesConfig},
    keys::ConsensusKeyPair,
    trusted_peers::ConsensusPeersConfig,
};
use failure::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConsensusConfig {
    pub max_block_size: u64,
    pub proposer_type: ConsensusProposerType,
    pub contiguous_rounds: u32,
    pub max_pruned_blocks_in_mem: Option<u64>,
    pub pacemaker_initial_timeout_ms: Option<u64>,
    // consensus_keypair contains the node's consensus keypair.
    // it is filled later on from consensus_keypair_file.
    #[serde(skip)]
    pub consensus_keypair: ConsensusKeyPair,
    pub consensus_keypair_file: PathBuf,
    #[serde(skip)]
    pub consensus_peers: ConsensusPeersConfig,
    pub consensus_peers_file: PathBuf,
    pub safety_rules: SafetyRulesConfig,
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        ConsensusConfig {
            max_block_size: 100,
            proposer_type: ConsensusProposerType::MultipleOrderedProposers,
            contiguous_rounds: 2,
            max_pruned_blocks_in_mem: None,
            pacemaker_initial_timeout_ms: None,
            consensus_keypair: ConsensusKeyPair::default(),
            consensus_keypair_file: PathBuf::from("consensus_keypair.config.toml"),
            consensus_peers: ConsensusPeersConfig::default(),
            consensus_peers_file: PathBuf::from("consensus_peers.config.toml"),
            safety_rules: SafetyRulesConfig::default(),
        }
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
}

impl ConsensusConfig {
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if !self.consensus_keypair_file.as_os_str().is_empty() {
            self.consensus_keypair = ConsensusKeyPair::load_config(
                path.as_ref().with_file_name(&self.consensus_keypair_file),
            );
        }
        if !self.consensus_peers_file.as_os_str().is_empty() {
            self.consensus_peers = ConsensusPeersConfig::load_config(
                path.as_ref().with_file_name(&self.consensus_peers_file),
            );
        }
        if let SafetyRulesBackend::OnDiskStorage {
            default,
            path: sr_path,
        } = &self.safety_rules.backend
        {
            // If the file is relative, it means it is in the same directory as this config,
            // unfortunately this is meaningless to the process that would load the config.
            if !sr_path.as_os_str().is_empty() && sr_path.is_relative() {
                self.safety_rules.backend = SafetyRulesBackend::OnDiskStorage {
                    default: *default,
                    path: path.as_ref().with_file_name(sr_path),
                };
            }
        }
        Ok(())
    }
}
