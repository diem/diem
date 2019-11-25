// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{BaseConfig, PersistableConfig, SafetyRulesConfig},
    keys::KeyPair,
    trusted_peers::{ConsensusPeerInfo, ConsensusPeersConfig},
};
use failure::Result;
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_types::PeerId;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

type ConsensusKeyPair = KeyPair<Ed25519PrivateKey>;

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
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        let keypair = ConsensusKeyPair::default();
        let peers = Self::default_peers(&keypair, PeerId::default());

        ConsensusConfig {
            max_block_size: 100,
            proposer_type: ConsensusProposerType::MultipleOrderedProposers,
            contiguous_rounds: 2,
            max_pruned_blocks_in_mem: None,
            pacemaker_initial_timeout_ms: None,
            consensus_keypair: keypair,
            consensus_keypair_file: PathBuf::new(),
            consensus_peers: peers,
            consensus_peers_file: PathBuf::new(),
            safety_rules: SafetyRulesConfig::default(),
            base: Arc::new(BaseConfig::default()),
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
    /// This clones the underlying data except for the keypair so that this config can be used as a
    /// template for another config.
    pub fn clone_for_template(&self) -> Self {
        Self {
            max_block_size: self.max_block_size,
            proposer_type: self.proposer_type,
            contiguous_rounds: self.contiguous_rounds,
            max_pruned_blocks_in_mem: self.max_pruned_blocks_in_mem,
            pacemaker_initial_timeout_ms: self.pacemaker_initial_timeout_ms,
            consensus_keypair: ConsensusKeyPair::default(),
            consensus_keypair_file: self.consensus_keypair_file.clone(),
            consensus_peers: self.consensus_peers.clone(),
            consensus_peers_file: self.consensus_peers_file.clone(),
            safety_rules: self.safety_rules.clone(),
            base: self.base.clone(),
        }
    }

    pub fn random(&mut self, rng: &mut StdRng, peer_id: PeerId) {
        let privkey = Ed25519PrivateKey::generate_for_testing(rng);
        let consensus_keypair = ConsensusKeyPair::load(privkey);
        self.consensus_peers = Self::default_peers(&consensus_keypair, peer_id);
        self.consensus_keypair = consensus_keypair;
    }

    pub fn prepare(&mut self, base: Arc<BaseConfig>) {
        self.base = base;
        self.safety_rules.prepare(self.base.clone());
    }

    pub fn load(&mut self) -> Result<()> {
        if !self.consensus_keypair_file.as_os_str().is_empty() {
            self.consensus_keypair = ConsensusKeyPair::load_config(self.consensus_keypair_file());
        }
        if !self.consensus_peers_file.as_os_str().is_empty() {
            self.consensus_peers = ConsensusPeersConfig::load_config(self.consensus_peers_file());
        }
        Ok(())
    }

    pub fn save(&mut self) {
        if self.consensus_keypair != ConsensusKeyPair::default() {
            if self.consensus_keypair_file.as_os_str().is_empty() {
                self.consensus_keypair_file = PathBuf::from("consensus.keys.toml");
            }

            self.consensus_keypair
                .save_config(self.consensus_keypair_file());
        }

        if self.consensus_peers_file.as_os_str().is_empty() {
            self.consensus_peers_file = PathBuf::from("consensus_peers.config.toml");
        }
        self.consensus_peers
            .save_config(self.consensus_peers_file());
    }

    pub fn consensus_keypair_file(&self) -> PathBuf {
        self.base.full_path(&self.consensus_keypair_file)
    }

    pub fn consensus_peers_file(&self) -> PathBuf {
        self.base.full_path(&self.consensus_peers_file)
    }

    fn default_peers(keypair: &ConsensusKeyPair, peer_id: PeerId) -> ConsensusPeersConfig {
        let mut peers = ConsensusPeersConfig::default();
        peers.peers.insert(
            peer_id,
            ConsensusPeerInfo {
                consensus_pubkey: keypair.public().clone(),
            },
        );
        peers
    }
}
