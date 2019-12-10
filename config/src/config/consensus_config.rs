// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{NetworkPeersConfig, PersistableConfig, RootPath, SafetyRulesConfig},
    keys::{self, KeyPair},
    utils,
};
use anyhow::Result;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    Uniform,
};
use libra_types::{
    crypto_proxies::{ValidatorInfo, ValidatorPublicKeys, ValidatorSet, ValidatorVerifier},
    PeerId,
};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

type ConsensusKeyPair = KeyPair<Ed25519PrivateKey>;

const CONSENSUS_KEYPAIR_DEFAULT: &str = "consensus.keys.toml";
const CONSENSUS_PEERS_DEFAULT: &str = "consensus_peers.config.toml";

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
        }
    }

    pub fn random(&mut self, rng: &mut StdRng, peer_id: PeerId) {
        let privkey = Ed25519PrivateKey::generate_for_testing(rng);
        let consensus_keypair = ConsensusKeyPair::load(privkey);
        self.consensus_peers = Self::default_peers(&consensus_keypair, peer_id);
        self.consensus_keypair = consensus_keypair;
    }

    pub fn load(&mut self, root_dir: &RootPath) -> Result<()> {
        if !self.consensus_keypair_file.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.consensus_keypair_file);
            self.consensus_keypair = ConsensusKeyPair::load_config(path)?;
        }
        if !self.consensus_peers_file.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.consensus_peers_file);
            self.consensus_peers = ConsensusPeersConfig::load_config(path)?;
        }
        Ok(())
    }

    pub fn save(&mut self, root_dir: &RootPath) -> Result<()> {
        if self.consensus_keypair != ConsensusKeyPair::default() {
            if self.consensus_keypair_file.as_os_str().is_empty() {
                self.consensus_keypair_file = PathBuf::from(CONSENSUS_KEYPAIR_DEFAULT);
            }

            let path = root_dir.full_path(&self.consensus_keypair_file);
            self.consensus_keypair.save_config(&path)?;
        }

        if self.consensus_peers_file.as_os_str().is_empty() {
            self.consensus_peers_file = PathBuf::from(CONSENSUS_PEERS_DEFAULT);
        }
        let path = root_dir.full_path(&self.consensus_peers_file);
        self.consensus_peers.save_config(path)?;
        Ok(())
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

#[cfg(test)]
mod test {
    use super::*;
    use libra_tools::tempdir::TempPath;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_with_defaults() {
        let keypair = ConsensusKeyPair::default();
        let peers = ConsensusConfig::default_peers(&keypair, PeerId::default());

        // Assert default exists
        let (mut config, path) = generate_config();
        assert_eq!(config.consensus_keypair, keypair);
        assert_eq!(config.consensus_keypair_file, PathBuf::new());
        assert_eq!(config.consensus_peers, peers);
        assert_eq!(config.consensus_peers_file, PathBuf::new());

        // Assert default loading doesn't affect paths and default remain in place
        let root_dir = RootPath::new_path(path.path());
        config.load(&root_dir).unwrap();
        assert_eq!(config.consensus_keypair, keypair);
        assert_eq!(config.consensus_keypair_file, PathBuf::new());
        assert_eq!(config.consensus_peers, peers);
        assert_eq!(config.consensus_peers_file, PathBuf::new());

        // Assert saving updates peers but not key pairs due to default behavior
        config.save(&root_dir).unwrap();
        assert_eq!(config.consensus_keypair, keypair);
        assert_eq!(config.consensus_keypair_file, PathBuf::new());
        assert_eq!(config.consensus_peers, peers);
        assert_eq!(
            config.consensus_peers_file,
            PathBuf::from(CONSENSUS_PEERS_DEFAULT)
        );
    }

    #[test]
    fn test_with_random() {
        let (mut config, path) = generate_config();
        let mut rng = StdRng::from_seed([6u8; 32]);
        config.random(&mut rng, PeerId::random());

        let keypair = config.consensus_keypair.clone();
        let peers = config.consensus_peers.clone();

        // Assert empty paths
        assert_eq!(config.consensus_keypair_file, PathBuf::new());
        assert_eq!(config.consensus_peers_file, PathBuf::new());

        // Assert saving updates paths
        let root_dir = RootPath::new_path(path.path());
        config.save(&root_dir).unwrap();
        assert_eq!(config.consensus_keypair, keypair);
        assert_eq!(
            config.consensus_keypair_file,
            PathBuf::from(CONSENSUS_KEYPAIR_DEFAULT)
        );
        assert_eq!(config.consensus_peers, peers);
        assert_eq!(
            config.consensus_peers_file,
            PathBuf::from(CONSENSUS_PEERS_DEFAULT)
        );

        // Assert a fresh load correctly populates the config
        let mut new_config = ConsensusConfig::default();
        // First that paths are empty
        assert_eq!(new_config.consensus_keypair_file, PathBuf::new());
        assert_eq!(new_config.consensus_peers_file, PathBuf::new());
        // Now set them to the defaults
        new_config.consensus_keypair_file = PathBuf::from(CONSENSUS_KEYPAIR_DEFAULT);
        new_config.consensus_peers_file = PathBuf::from(CONSENSUS_PEERS_DEFAULT);
        // Loading populates things correctly
        new_config.load(&root_dir).unwrap();
        assert_eq!(new_config.consensus_keypair, keypair);
        assert_eq!(
            new_config.consensus_keypair_file,
            PathBuf::from(CONSENSUS_KEYPAIR_DEFAULT)
        );
        assert_eq!(new_config.consensus_peers, peers);
        assert_eq!(
            new_config.consensus_peers_file,
            PathBuf::from(CONSENSUS_PEERS_DEFAULT)
        );
    }

    fn generate_config() -> (ConsensusConfig, TempPath) {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        let consensus_config = ConsensusConfig::default();
        (consensus_config, temp_dir)
    }
}
