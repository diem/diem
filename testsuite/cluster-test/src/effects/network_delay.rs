// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// NetworkDelay introduces network delay from a given instance to a provided list of instances
/// If no instances are provided, network delay is introduced on all outgoing packets
use crate::{effects::Action, instance::Instance};
use failure;
use slog_scope::info;
use std::fmt;
use std::time::Duration;

pub struct NetworkDelay {
    instance: Instance,
    target_instances: Option<Vec<Instance>>,
    delay: Duration,
}

impl NetworkDelay {
    pub fn new(
        instance: Instance,
        target_instances: Option<Vec<Instance>>,
        delay: Duration,
    ) -> Self {
        Self {
            instance,
            target_instances,
            delay,
        }
    }
}

impl Action for NetworkDelay {
    fn apply(&self) -> failure::Result<()> {
        info!(
            "NetworkDelay {}ms for {}",
            self.delay.as_millis(),
            self.instance
        );
        let mut command = "".to_string();
        if let Some(target_instances) = &self.target_instances {
            command += "sudo tc qdisc delete dev eth0 root; ";
            // Create a HTB https://linux.die.net/man/8/tc-htb
            command += "sudo tc qdisc add dev eth0 root handle 1: htb; ";
            // Create a class within the HTB https://linux.die.net/man/8/tc
            command += "sudo tc class add dev eth0 parent 1: classid 1:1 htb rate 1tbit; ";
            // Create u32 filters so that all the target instances are classified as class 1:1
            // http://man7.org/linux/man-pages/man8/tc-u32.8.html
            for target_instance in target_instances {
                command += format!("sudo tc filter add dev eth0 parent 1: protocol ip prio 1 u32 flowid 1:1 match ip dst {}; ", target_instance.ip()).as_str();
            }
            // Use netem to delay packets to this class
            command += format!(
                "sudo tc qdisc add dev eth0 parent 1:1 handle 10: netem delay {}ms; ",
                self.delay.as_millis()
            )
            .as_str();
        } else {
            // If there are no target instances, add netem to the root qdisc
            command = format!("sudo tc qdisc delete dev eth0 root; sudo tc qdisc add dev eth0 root netem delay {}ms; ", self.delay.as_millis());
        }
        self.instance.run_cmd(vec![command])
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl fmt::Display for NetworkDelay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NetworkDelay {}ms from {}",
            self.delay.as_millis(),
            self.instance
        )
    }
}
