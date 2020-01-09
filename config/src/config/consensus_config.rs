// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{NetworkPeersConfig, PersistableConfig, RootPath, SafetyRulesConfig},
    keys, utils,
};
use anyhow::Result;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_types::{
    crypto_proxies::{ValidatorInfo, ValidatorPublicKeys, ValidatorSet, ValidatorVerifier},
    PeerId,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

const CONSENSUS_PEERS_DEFAULT: &str = "consensus_peers.config.toml";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConsensusConfig {
    pub max_block_size: u64,
    pub proposer_type: ConsensusProposerType,
    pub contiguous_rounds: u32,
    pub max_pruned_blocks_in_mem: Option<u64>,
    pub pacemaker_initial_timeout_ms: Option<u64>,
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
            consensus_peers: ConsensusPeersConfig::default(),
            consensus_peers_file: PathBuf::new(),
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
    pub fn load(&mut self, root_dir: &RootPath) -> Result<()> {
        if !self.consensus_peers_file.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.consensus_peers_file);
            self.consensus_peers = ConsensusPeersConfig::load_config(path)?;
        }
        Ok(())
    }

    pub fn save(&mut self, root_dir: &RootPath) -> Result<()> {
        if self.consensus_peers_file.as_os_str().is_empty() {
            self.consensus_peers_file = PathBuf::from(CONSENSUS_PEERS_DEFAULT);
        }
        let path = root_dir.full_path(&self.consensus_peers_file);
        self.consensus_peers.save_config(path)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Deserialize)]
pub struct ConsensusPeersConfig {
    #[serde(flatten)]
    #[serde(serialize_with = "utils::serialize_ordered_map")]
    pub peers: HashMap<PeerId, ConsensusPeerInfo>,
}

impl ConsensusPeersConfig {
    /// Return a sorted vector of ValidatorPublicKey's
    pub fn get_validator_set(&self, network_peers_config: &NetworkPeersConfig) -> ValidatorSet {
        let mut keys: Vec<ValidatorPublicKeys> = self
            .peers
            .iter()
            .map(|(peer_id, peer_info)| {
                ValidatorPublicKeys::new(
                    *peer_id,
                    peer_info.consensus_pubkey.clone(),
                    // TODO: Add support for dynamic voting weights in config
                    1,
                    network_peers_config
                        .peers
                        .get(peer_id)
                        .unwrap()
                        .signing_public_key
                        .clone(),
                    network_peers_config
                        .peers
                        .get(peer_id)
                        .unwrap()
                        .identity_public_key
                        .clone(),
                )
            })
            .collect();
        // self.peers is a HashMap, so iterating over it produces a differently ordered vector each
        // time. Sort by account address to produce a canonical ordering
        keys.sort_by(|k1, k2| k1.account_address().cmp(k2.account_address()));
        ValidatorSet::new(keys)
    }

    pub fn get_validator_verifier(&self) -> ValidatorVerifier {
        ValidatorVerifier::new(
            self.peers
                .iter()
                .map(|(peer_id, peer_info)| {
                    (
                        *peer_id,
                        ValidatorInfo::new(peer_info.consensus_pubkey.clone(), 1),
                    )
                })
                .collect(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConsensusPeerInfo {
    #[serde(serialize_with = "keys::serialize_key")]
    #[serde(deserialize_with = "keys::deserialize_key")]
    #[serde(rename = "c")]
    pub consensus_pubkey: Ed25519PublicKey,
}
