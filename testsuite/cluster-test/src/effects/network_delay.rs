// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::effects::Effect;
/// NetworkDelay introduces network delay from a given instance to a provided list of instances
/// If no instances are provided, network delay is introduced on all outgoing packets
use crate::instance::Instance;
use anyhow::Result;

use async_trait::async_trait;
use slog_scope::debug;
use std::fmt;
use std::time::Duration;

pub struct NetworkDelay {
    instance: Instance,
    // A vector of a pair of (delay, instance list)
    // Applies delay to each instance in the instance list
    configuration: Vec<(Vec<Instance>, Duration)>,
}

impl NetworkDelay {
    pub fn new(instance: Instance, configuration: Vec<(Vec<Instance>, Duration)>) -> Self {
        Self {
            instance,
            configuration,
        }
    }
}

#[async_trait]
impl Effect for NetworkDelay {
    async fn activate(&self) -> Result<()> {
        debug!("Injecting NetworkDelays for {}", self.instance);
        let mut command = "".to_string();
        command += "sudo tc qdisc delete dev eth0 root; ";
        // Create a HTB https://linux.die.net/man/8/tc-htb
        command += "sudo tc qdisc add dev eth0 root handle 1: htb; ";
        for i in 0..self.configuration.len() {
            // Create a class within the HTB https://linux.die.net/man/8/tc
            command += format!(
                "sudo tc class add dev eth0 parent 1: classid 1:{} htb rate 1tbit; ",
                i + 1
            )
            .as_str();
        }
        for i in 0..self.configuration.len() {
            // Create u32 filters so that all the target instances are classified as class 1:(i+1)
            // http://man7.org/linux/man-pages/man8/tc-u32.8.html
            for target_instance in &self.configuration[i].0 {
                command += format!("sudo tc filter add dev eth0 parent 1: protocol ip prio 1 u32 flowid 1:{} match ip dst {}; ", i+1, target_instance.ip()).as_str();
            }
        }
        for i in 0..self.configuration.len() {
            // Use netem to delay packets to this class
            command += format!(
                "sudo tc qdisc add dev eth0 parent 1:{} handle {}0: netem delay {}ms; ",
                i + 1,
                i + 1,
                self.configuration[i].1.as_millis(),
            )
            .as_str();
        }
        self.instance.run_cmd(vec![command]).await
    }

    async fn deactivate(&self) -> Result<()> {
        self.instance
            .run_cmd(vec!["sudo tc qdisc delete dev eth0 root; true".to_string()])
            .await
    }
}

impl fmt::Display for NetworkDelay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NetworkDelay from {}", self.instance)
    }
}

/// three_region_simulation_effects returns the list of NetworkDelays which need to be applied to
/// all the instances in the cluster.
/// `regions` is a 3-tuple consisting of the list of instances in each region
/// `delays_bw_regions` is a 3-tuple consisting of the one-way delays between pairs of regions
/// delays_bw_regions.0 is the delay b/w regions 1 & 2, delays_bw_regions.1 is the delay b/w regions 0 & 2, etc
pub fn three_region_simulation_effects(
    regions: (Vec<Instance>, Vec<Instance>, Vec<Instance>),
    delays_bw_regions: (Duration, Duration, Duration),
) -> Vec<NetworkDelay> {
    let mut result = vec![];
    for instance in &regions.0 {
        let configuration = vec![
            (regions.1.clone(), delays_bw_regions.2),
            (regions.2.clone(), delays_bw_regions.1),
        ];
        result.push(NetworkDelay::new(instance.clone(), configuration));
    }
    for instance in &regions.1 {
        let configuration = vec![
            (regions.0.clone(), delays_bw_regions.2),
            (regions.2.clone(), delays_bw_regions.0),
        ];
        result.push(NetworkDelay::new(instance.clone(), configuration));
    }
    for instance in &regions.2 {
        let configuration = vec![
            (regions.1.clone(), delays_bw_regions.0),
            (regions.0.clone(), delays_bw_regions.1),
        ];
        result.push(NetworkDelay::new(instance.clone(), configuration));
    }
    result
}
