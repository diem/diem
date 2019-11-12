// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ConsensusProposerType::{FixedProposer, MultipleOrderedProposers, RotatingProposer},
    config::{PersistableConfig, SafetyRulesBackend, SafetyRulesConfig},
    keys::ConsensusKeyPair,
    trusted_peers::ConsensusPeersConfig,
};
use failure::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsensusConfig {
    pub max_block_size: u64,
    pub proposer_type: String,
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
            proposer_type: "multiple_ordered_proposers".to_string(),
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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

    pub fn get_proposer_type(&self) -> ConsensusProposerType {
        match self.proposer_type.as_str() {
            "fixed_proposer" => FixedProposer,
            "rotating_proposer" => RotatingProposer,
            "multiple_ordered_proposers" => MultipleOrderedProposers,
            &_ => unimplemented!("Invalid proposer type: {}", self.proposer_type),
        }
    }

    pub fn contiguous_rounds(&self) -> u32 {
        self.contiguous_rounds
    }

    pub fn max_block_size(&self) -> u64 {
        self.max_block_size
    }

    pub fn max_pruned_blocks_in_mem(&self) -> &Option<u64> {
        &self.max_pruned_blocks_in_mem
    }

    pub fn pacemaker_initial_timeout_ms(&self) -> &Option<u64> {
        &self.pacemaker_initial_timeout_ms
    }

    pub fn safety_rules(&self) -> &SafetyRulesConfig {
        &self.safety_rules
    }
}
