use crate::{aws::Aws, instance::Instance};
use failure::{self, prelude::*};
use rand::prelude::*;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Filter};
use std::{thread, time::Duration};

#[derive(Clone)]
pub struct Cluster {
    instances: Vec<Instance>, // guaranteed non-empty
}

impl Cluster {
    pub fn discover(aws: &Aws) -> failure::Result<Self> {
        let mut instances = vec![];
        let mut next_token = None;
        let mut retries_left = 10;
        loop {
            let filters = vec![
                Filter {
                    name: Some("tag:Workspace".into()),
                    values: Some(vec![aws.workplace().clone()]),
                },
                Filter {
                    name: Some("tag:Role".into()),
                    values: Some(vec!["validator".into()]),
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
                    let peer_id = tags
                        .into_iter()
                        .find(|tag| tag.key == Some("PeerId".into()))
                        .expect("No peer id")
                        .value
                        .expect("PeerId tag has no value");
                    let short_hash = peer_id[..8].into();
                    instances.push(Instance::new(short_hash, ip));
                }
            }
            next_token = result.next_token;
            if next_token.is_none() {
                break;
            }
        }
        ensure!(
            !instances.is_empty(),
            "Non instances were discovered for cluster"
        );
        Ok(Self { instances })
    }

    pub fn random_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.instances.choose(&mut rnd).unwrap().clone()
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }

    pub fn get_instance(&self, name: &str) -> Option<&Instance> {
        for instance in &self.instances {
            if instance.short_hash() == name {
                return Some(instance);
            }
        }
        None
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
        Cluster { instances }
    }
}
