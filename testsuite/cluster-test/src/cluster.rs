// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{aws::Aws, instance::Instance};
use anyhow::{ensure, format_err, Result};
use config_builder::ValidatorConfig;
use generate_keypair::load_key_from_file;
use libra_config::config::AdmissionControlConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use rand::prelude::*;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Filter, Tag};
use slog_scope::*;
use std::{collections::HashMap, convert::TryInto, thread, time::Duration};

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
        let mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
            load_key_from_file(mint_file).expect("invalid faucet keypair file");
        Self {
            validator_instances: instances,
            fullnode_instances: vec![],
            prometheus_ip: None,
            mint_key_pair,
        }
    }

    pub fn discover(aws: &Aws) -> Result<Self> {
        let mut validator_instances = vec![];
        let mut fullnode_instances = vec![];
        let mut next_token = None;
        let mut retries_left = 10;
        let mut prometheus_ip: Option<String> = None;
        loop {
            let filters = vec![
                Filter {
                    name: Some("tag:Workspace".into()),
                    values: Some(vec![aws.workspace().clone()]),
                },
                Filter {
                    name: Some("instance-state-name".into()),
                    values: Some(vec!["running".into()]),
                },
            ];
            let result = aws
                .ec2()
                .describe_instances(DescribeInstancesRequest {
                    filters: Some(filters),
                    max_results: Some(1000),
                    dry_run: None,
                    instance_ids: None,
                    next_token: next_token.clone(),
                })
                .sync();
            let result = match result {
                Err(e) => {
                    warn!(
                        "Failed to describe aws instances: {:?}, retries left: {}",
                        e, retries_left
                    );
                    thread::sleep(Duration::from_secs(1));
                    if retries_left == 0 {
                        panic!("Last attempt to describe instances failed");
                    }
                    retries_left -= 1;
                    continue;
                }
                Ok(r) => r,
            };
            let ac_port = AdmissionControlConfig::default().address.port() as u32;
            for reservation in result.reservations.expect("no reservations") {
                for aws_instance in reservation.instances.expect("no instances") {
                    let ip = aws_instance
                        .private_ip_address
                        .expect("Instance does not have private IP address");
                    let tags = aws_instance.tags.expect("Instance does not have tags");
                    let role = parse_tags(tags);
                    match role {
                        InstanceRole::Prometheus => {
                            prometheus_ip = Some(ip);
                        }
                        InstanceRole::Validator(peer_name) => {
                            validator_instances.push(Instance::new(peer_name, ip, ac_port));
                        }
                        InstanceRole::Fullnode(peer_name) => {
                            fullnode_instances.push(Instance::new(peer_name, ip, ac_port));
                        }
                        _ => {}
                    }
                }
            }
            next_token = result.next_token;
            if next_token.is_none() {
                break;
            }
        }
        ensure!(
            !validator_instances.is_empty(),
            "No instances were discovered for cluster"
        );
        let prometheus_ip =
            prometheus_ip.ok_or_else(|| format_err!("Prometheus was not found in workspace"))?;
        let seed = "1337133713371337133713371337133713371337133713371337133713371337";
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        let seed = seed[..32].try_into().expect("Invalid seed");
        let mint_key = ValidatorConfig::new().seed(seed).build_faucet_client();
        let mint_key_pair = KeyPair::from(mint_key);
        Ok(Self {
            validator_instances,
            fullnode_instances,
            prometheus_ip: Some(prometheus_ip),
            mint_key_pair,
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

fn parse_tags(tags: Vec<Tag>) -> InstanceRole {
    let mut map: HashMap<_, _> = tags.into_iter().map(|tag| (tag.key, tag.value)).collect();
    let role = map.remove(&Some("Role".to_string()));
    if role == Some(Some("validator".to_string())) {
        let peer_name = map.remove(&Some("Name".to_string()));
        let peer_name = peer_name.expect("Validator instance without Name");
        let peer_name = peer_name.expect("'Name' tag without value");
        return InstanceRole::Validator(peer_name);
    } else if role == Some(Some("fullnode".to_string())) {
        let peer_name = map.remove(&Some("Name".to_string()));
        let peer_name = peer_name.expect("Fullnode instance without Name");
        let peer_name = peer_name.expect("'Name' tag without value");
        return InstanceRole::Fullnode(peer_name);
    } else if role == Some(Some("monitoring".to_string())) {
        return InstanceRole::Prometheus;
    }
    InstanceRole::Unknown
}

enum InstanceRole {
    Validator(String),
    Fullnode(String),
    Prometheus,
    Unknown,
}
