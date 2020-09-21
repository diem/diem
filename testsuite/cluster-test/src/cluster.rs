// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::instance::{Instance, ValidatorGroup};
use config_builder::ValidatorConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    Uniform,
};
use libra_types::{chain_id::ChainId, waypoint::Waypoint};
use rand::prelude::*;
use reqwest::Client;
use std::convert::TryInto;

#[derive(Clone)]
pub struct Cluster {
    // guaranteed non-empty
    validator_instances: Vec<Instance>,
    fullnode_instances: Vec<Instance>,
    lsr_instances: Vec<Instance>,
    vault_instances: Vec<Instance>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    waypoint: Option<Waypoint>,
    pub chain_id: ChainId,
}

pub fn dummy_key_pair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    Ed25519PrivateKey::generate_for_testing().into()
}

impl Cluster {
    pub fn from_host_port(
        peers: Vec<(String, u32, Option<u32>)>,
        mint_file: &str,
        chain_id: ChainId,
        premainnet: bool,
    ) -> Self {
        let http_client = Client::new();
        let instances: Vec<Instance> = peers
            .into_iter()
            .map(|host_port| {
                Instance::new(
                    format!("{}:{}", &host_port.0, host_port.1), /* short_hash */
                    host_port.0,
                    host_port.1,
                    host_port.2,
                    http_client.clone(),
                )
            })
            .collect();

        let mint_key_pair = if premainnet {
            dummy_key_pair()
        } else {
            KeyPair::from(generate_key::load_key(mint_file))
        };
        Self {
            validator_instances: instances,
            fullnode_instances: vec![],
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair,
            waypoint: None,
            chain_id,
        }
    }

    fn get_mint_key_pair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        let seed = "1337133713371337133713371337133713371337133713371337133713371337";
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        let seed = seed[..32].try_into().expect("Invalid seed");
        let mut validator_config = ValidatorConfig::new();
        validator_config.seed = seed;
        let (mint_key, _) = validator_config.build_libra_root_key();
        KeyPair::from(mint_key)
    }

    fn get_mint_key_pair_from_file(
        mint_file: &str,
    ) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        let mint_key: Ed25519PrivateKey = generate_key::load_key(mint_file);
        KeyPair::from(mint_key)
    }

    pub fn new(
        validator_instances: Vec<Instance>,
        fullnode_instances: Vec<Instance>,
        lsr_instances: Vec<Instance>,
        vault_instances: Vec<Instance>,
        waypoint: Option<Waypoint>,
    ) -> Self {
        let mint_key_pair = if waypoint.is_some() {
            Self::get_mint_key_pair_from_file("/tmp/mint.key")
        } else {
            Self::get_mint_key_pair()
        };
        Self {
            validator_instances,
            fullnode_instances,
            lsr_instances,
            vault_instances,
            mint_key_pair,
            waypoint,
            chain_id: ChainId::test(),
        }
    }

    pub fn random_validator_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.validator_instances
            .choose(&mut rnd)
            .expect("random_validator_instance requires non-empty validator_instances")
            .clone()
    }

    pub fn validator_instances(&self) -> &[Instance] {
        &self.validator_instances
    }

    pub fn random_fullnode_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.fullnode_instances
            .choose(&mut rnd)
            .expect("random_full_node_instance requires non-empty fullnode_instances")
            .clone()
    }

    pub fn fullnode_instances(&self) -> &[Instance] {
        &self.fullnode_instances
    }

    pub fn lsr_instances(&self) -> &[Instance] {
        &self.lsr_instances
    }

    pub fn vault_instances(&self) -> &[Instance] {
        &self.vault_instances
    }

    pub fn all_instances(&self) -> impl Iterator<Item = &Instance> {
        self.validator_instances
            .iter()
            .chain(self.fullnode_instances.iter())
            .chain(self.lsr_instances.iter())
            .chain(self.vault_instances.iter())
    }

    pub fn validator_and_fullnode_instances(&self) -> impl Iterator<Item = &Instance> {
        self.validator_instances
            .iter()
            .chain(self.fullnode_instances.iter())
    }

    pub fn into_validator_instances(self) -> Vec<Instance> {
        self.validator_instances
    }

    pub fn into_fullnode_instances(self) -> Vec<Instance> {
        self.fullnode_instances
    }

    pub fn into_lsr_instances(self) -> Vec<Instance> {
        self.lsr_instances
    }

    pub fn into_vault_instances(self) -> Vec<Instance> {
        self.vault_instances
    }

    pub fn mint_key_pair(&self) -> &KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        &self.mint_key_pair
    }

    pub fn get_validator_instance(&self, name: &str) -> Option<&Instance> {
        self.validator_instances
            .iter()
            .find(|instance| instance.peer_name() == name)
    }

    /// Splits this cluster into two
    ///
    /// Returns tuple of two clusters:
    /// First element in tuple contains cluster with c random instances from self
    /// Second element in tuple contains cluster with remaining instances from self
    pub fn split_n_validators_random(&self, c: usize) -> (Self, Self) {
        assert!(c <= self.validator_instances.len());
        let mut rng = ThreadRng::default();
        let mut sub = vec![];
        let mut rem = self.validator_instances.clone();
        for _ in 0..c {
            let idx_remove = rng.gen_range(0, rem.len());
            let instance = rem.remove(idx_remove);
            sub.push(instance);
        }
        (
            self.new_validator_sub_cluster(sub),
            self.new_validator_sub_cluster(rem),
        )
    }

    pub fn split_n_fullnodes_random(&self, c: usize) -> (Self, Self) {
        assert!(c <= self.fullnode_instances.len());
        let mut rng = ThreadRng::default();
        let mut sub = vec![];
        let mut rem = self.fullnode_instances.clone();
        for _ in 0..c {
            let idx_remove = rng.gen_range(0, rem.len());
            let instance = rem.remove(idx_remove);
            sub.push(instance);
        }
        (
            self.new_fullnode_sub_cluster(sub),
            self.new_fullnode_sub_cluster(rem),
        )
    }

    fn new_validator_sub_cluster(&self, instances: Vec<Instance>) -> Self {
        Cluster {
            validator_instances: instances,
            fullnode_instances: vec![],
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair: self.mint_key_pair.clone(),
            waypoint: self.waypoint,
            chain_id: ChainId::test(),
        }
    }

    fn new_fullnode_sub_cluster(&self, instances: Vec<Instance>) -> Self {
        Cluster {
            validator_instances: vec![],
            fullnode_instances: instances,
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair: self.mint_key_pair.clone(),
            waypoint: self.waypoint,
            chain_id: ChainId::test(),
        }
    }

    pub fn validator_sub_cluster(&self, ids: Vec<String>) -> Cluster {
        let mut instances = Vec::with_capacity(ids.len());
        for id in ids {
            let instance = self.get_validator_instance(&id);
            match instance {
                Some(instance) => instances.push(instance.clone()),
                None => panic!("Can not make sub_cluster: instance {} is not found", id),
            }
        }
        assert!(!instances.is_empty(), "No instances for subcluster");
        self.new_validator_sub_cluster(instances)
    }

    pub fn find_instance_by_pod(&self, pod: &str) -> Option<&Instance> {
        self.validator_and_fullnode_instances()
            .find(|i| i.peer_name() == pod)
    }

    pub fn instances_for_group(
        &self,
        validator_group: ValidatorGroup,
    ) -> impl Iterator<Item = &Instance> {
        self.all_instances()
            .filter(move |v| v.validator_group() == validator_group)
    }

    pub fn lsr_instances_for_validators(&self, validators: &[Instance]) -> Vec<Instance> {
        validators
            .iter()
            .filter_map(|l| {
                self.lsr_instances
                    .iter()
                    .find(|x| l.validator_group() == x.validator_group())
                    .cloned()
            })
            .collect()
    }

    pub fn vault_instances_for_validators(&self, validators: &[Instance]) -> Vec<Instance> {
        validators
            .iter()
            .filter_map(|v| {
                self.vault_instances
                    .iter()
                    .find(|x| v.validator_group() == x.validator_group())
                    .cloned()
            })
            .collect()
    }
}
