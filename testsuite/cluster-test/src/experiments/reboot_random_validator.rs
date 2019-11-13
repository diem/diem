// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    effects::{Action, Reboot},
    experiments::Experiment,
    instance::Instance,
};
use failure;
use rand::Rng;
use std::{collections::HashSet, fmt, thread, time::Duration};

pub struct RebootRandomValidators {
    instances: Vec<Instance>,
}

impl RebootRandomValidators {
    pub fn new(count: usize, cluster: &Cluster) -> Self {
        if count > cluster.instances().len() {
            panic!(
                "Can not reboot {} validators in cluster with {} instances",
                count,
                cluster.instances().len()
            );
        }
        let mut instances = Vec::with_capacity(count);
        let mut all_instances = cluster.instances().clone();
        let mut rnd = rand::thread_rng();
        for _i in 0..count {
            let instance = all_instances.remove(rnd.gen_range(0, all_instances.len()));
            instances.push(instance);
        }

        Self { instances }
    }
}

impl Experiment for RebootRandomValidators {
    fn affected_validators(&self) -> HashSet<String> {
        let mut r = HashSet::new();
        for instance in self.instances.iter() {
            r.insert(instance.short_hash().clone());
        }
        r
    }

    fn run(&self) -> failure::Result<()> {
        let mut reboots = vec![];
        for instance in self.instances.iter() {
            let reboot = Reboot::new(instance.clone());
            reboot.apply()?;
            reboots.push(reboot)
        }
        while reboots.iter().any(|r| !r.is_complete()) {
            thread::sleep(Duration::from_secs(5));
        }
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(20 * 60)
    }
}

impl fmt::Display for RebootRandomValidators {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reboot [")?;
        for instance in self.instances.iter() {
            write!(f, "{}, ", instance)?;
        }
        write!(f, "]")
    }
}
