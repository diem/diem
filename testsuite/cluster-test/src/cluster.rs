// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{aws::Aws, instance::Instance};
use failure::{self, prelude::*};
use rand::prelude::*;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Filter, Tag};
use std::collections::HashMap;
use std::{thread, time::Duration};

#[derive(Clone)]
pub struct Cluster {
    // guaranteed non-empty
    instances: Vec<Instance>,
    prometheus_ip: Option<String>,
    mint_file: String,
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
        Self {
            instances,
            prometheus_ip: None,
            mint_file: mint_file.to_string(),
        }
    }

    pub fn discover(aws: &Aws, mint_file: &str) -> failure::Result<Self> {
        let mut instances = vec![];
        let mut next_token = None;
        let mut retries_left = 10;
        let mut prometheus_ip: Option<String> = None;
        loop {
            let filters = vec![
                Filter {
                    name: Some("tag:Workspace".into()),
                    values: Some(vec![aws.workplace().clone()]),
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
                    println!(
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
                        InstanceRole::Peer(peer_id) => {
                            let short_hash = peer_id[..8].into();
                            instances.push(Instance::new(short_hash, ip, 8000));
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
            !instances.is_empty(),
            "No instances were discovered for cluster"
        );
        let prometheus_ip =
            prometheus_ip.ok_or_else(|| format_err!("Prometheus was not found in workplace"))?;
        Ok(Self {
            instances,
            prometheus_ip: Some(prometheus_ip),
            mint_file: mint_file.to_string(),
        })
    }

    pub fn random_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.instances.choose(&mut rnd).unwrap().clone()
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }

    pub fn into_instances(self) -> Vec<Instance> {
        self.instances
    }

    pub fn prometheus_ip(&self) -> Option<&String> {
        self.prometheus_ip.as_ref()
    }

    pub fn mint_file(&self) -> &str {
        &self.mint_file
    }

    pub fn get_instance(&self, name: &str) -> Option<&Instance> {
        self.instances
            .iter()
            .find(|instance| instance.short_hash() == name)
    }

    /// Splits this cluster into two
    ///
    /// Returns tuple of two clusters:
    /// First element in tuple contains cluster with c random instances from self
    /// Second element in tuple contains cluster with remaining instances from self
    pub fn split_n_random(&self, c: usize) -> (Self, Self) {
        assert!(c <= self.instances.len());
        let mut rng = ThreadRng::default();
        let mut sub = vec![];
        let mut rem = self.instances.clone();
        for _ in 0..c {
            let idx_remove = rng.gen_range(0, rem.len());
            let instance = rem.remove(idx_remove);
            sub.push(instance);
        }
        (self.new_sub_cluster(sub), self.new_sub_cluster(rem))
    }

    fn new_sub_cluster(&self, instances: Vec<Instance>) -> Self {
        Cluster {
            instances,
            prometheus_ip: self.prometheus_ip.clone(),
            mint_file: self.mint_file.clone(),
        }
    }

    pub fn sub_cluster(&self, ids: Vec<String>) -> Cluster {
        let mut instances = Vec::with_capacity(ids.len());
        for id in ids {
            let instance = self.get_instance(&id);
            match instance {
                Some(instance) => instances.push(instance.clone()),
                None => panic!("Can not make sub_cluster: instance {} is not found", id),
            }
        }
        assert!(!instances.is_empty(), "No instances for subcluster");
        self.new_sub_cluster(instances)
    }
}

fn parse_tags(tags: Vec<Tag>) -> InstanceRole {
    let mut map: HashMap<_, _> = tags.into_iter().map(|tag| (tag.key, tag.value)).collect();
    let role = map.remove(&Some("Role".to_string()));
    if role == Some(Some("validator".to_string())) {
        let peer_id = map.remove(&Some("PeerId".to_string()));
        let peer_id = peer_id.expect("Validator instance without PeerId");
        let peer_id = peer_id.expect("PeerId tag without value");
        return InstanceRole::Peer(peer_id);
    } else if role == Some(Some("monitoring".to_string())) {
        return InstanceRole::Prometheus;
    }
    InstanceRole::Unknown
}

enum InstanceRole {
    Peer(String),
    Prometheus,
    Unknown,
}
