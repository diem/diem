// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{BaseConfig, PersistableConfig, SafetyRulesConfig},
    keys::ConsensusKeyPair,
    trusted_peers::{ConsensusPeerInfo, ConsensusPeersConfig},
};
use failure::Result;
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_types::PeerId;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

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
    pub fn random(rng: &mut StdRng, peer_id: PeerId) -> Self {
        let privkey = Ed25519PrivateKey::generate_for_testing(rng);
        let consensus_keypair = ConsensusKeyPair::load(Some(privkey));
        let mut config = Self::default();
        config.consensus_peers = Self::default_peers(&consensus_keypair, peer_id);
        config.consensus_keypair = consensus_keypair;
        config
    }

    pub fn load(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        self.base = base;
        if !self.consensus_keypair_file.as_os_str().is_empty() {
            self.consensus_keypair = ConsensusKeyPair::load_config(self.consensus_keypair_file());
        }
        if !self.consensus_peers_file.as_os_str().is_empty() {
            self.consensus_peers = ConsensusPeersConfig::load_config(self.consensus_peers_file());
        }
        self.safety_rules.load(self.base.clone())?;
        Ok(())
    }

    pub fn consensus_keypair_file(&self) -> PathBuf {
        self.base.full_path(&self.consensus_keypair_file)
    }

    pub fn consensus_peers_file(&self) -> PathBuf {
        self.base.full_path(&self.consensus_peers_file)
    }

    fn default_peers(keypair: &ConsensusKeyPair, peer_id: PeerId) -> ConsensusPeersConfig {
        let mut peers = ConsensusPeersConfig::default();
        let pubkey = keypair
            .public()
            .expect("Unable to obtain default public key");
        peers.peers.insert(
            peer_id.to_string(),
            ConsensusPeerInfo {
                consensus_pubkey: pubkey.clone(),
            },
        );
        peers
    }
}
