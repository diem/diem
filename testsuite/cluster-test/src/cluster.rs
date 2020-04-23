// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::instance::Instance;
use anyhow::Result;
use config_builder::ValidatorConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use rand::prelude::*;
use std::convert::TryInto;

#[derive(Clone)]
pub struct Cluster {
    // guaranteed non-empty
    validator_instances: Vec<Instance>,
    fullnode_instances: Vec<Instance>,
    prometheus_ip: Option<String>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
}

impl Cluster {
    pub fn from_host_port(peers: Vec<(String, u32)>, mint_file: &str) -> Self {
        let instances: Vec<Instance> = peers
            .into_iter()
            .map(|host_port| {
                Instance::new(
                    format!("{}:{}", &host_port.0, host_port.1), /* short_hash */
                    host_port.0,
                    host_port.1,
                )
            })
            .collect();
        let mint_key: Ed25519PrivateKey = generate_key::load_key(mint_file);
        let mint_key_pair = KeyPair::from(mint_key);
        Self {
            validator_instances: instances,
            fullnode_instances: vec![],
            prometheus_ip: None,
            mint_key_pair,
        }
    }

    fn get_mint_key_pair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        let seed = "1337133713371337133713371337133713371337133713371337133713371337";
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        let seed = seed[..32].try_into().expect("Invalid seed");
        let mint_key = ValidatorConfig::new().seed(seed).build_faucet_client();
        KeyPair::from(mint_key)
    }

    pub fn new_k8s(
        validator_instances: Vec<Instance>,
        fullnode_instances: Vec<Instance>,
    ) -> Result<Self> {
        Ok(Self {
            validator_instances,
            fullnode_instances,
            prometheus_ip: None,
            mint_key_pair: Self::get_mint_key_pair(),
        })
    }

    pub fn random_validator_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.validator_instances.choose(&mut rnd).unwrap().clone()
    }

    pub fn validator_instances(&self) -> &[Instance] {
        &self.validator_instances
    }
    pub fn fullnode_instances(&self) -> &[Instance] {
        &self.fullnode_instances
    }

    pub fn all_instances(&self) -> impl Iterator<Item = &Instance> {
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

    pub fn prometheus_ip(&self) -> Option<&String> {
        self.prometheus_ip.as_ref()
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
            prometheus_ip: self.prometheus_ip.clone(),
            mint_key_pair: self.mint_key_pair.clone(),
        }
    }

    fn new_fullnode_sub_cluster(&self, instances: Vec<Instance>) -> Self {
        Cluster {
            validator_instances: vec![],
            fullnode_instances: instances,
            prometheus_ip: self.prometheus_ip.clone(),
            mint_key_pair: self.mint_key_pair.clone(),
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
}
