// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, thread, time::Duration};

use rand::Rng;

use failure;

use crate::experiments::Context;
use crate::{
    cluster::Cluster,
    effects::{Action, Reboot},
    experiments::Experiment,
    instance,
    instance::Instance,
};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RebootRandomValidatorsParams {
    #[structopt(long, default_value = "10", help = "Number of nodes to reboot")]
    pub count: usize,
}

pub struct RebootRandomValidators {
    instances: Vec<Instance>,
}

impl RebootRandomValidators {
    pub fn new(params: RebootRandomValidatorsParams, cluster: &Cluster) -> Self {
        if params.count > cluster.instances().len() {
            panic!(
                "Can not reboot {} validators in cluster with {} instances",
                params.count,
                cluster.instances().len()
            );
        }
        let mut instances = Vec::with_capacity(params.count);
        let mut all_instances = cluster.instances().clone();
        let mut rnd = rand::thread_rng();
        for _i in 0..params.count {
            let instance = all_instances.remove(rnd.gen_range(0, all_instances.len()));
            instances.push(instance);
        }

        Self { instances }
    }
}

impl Experiment for RebootRandomValidators {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.instances)
    }

    fn run(&mut self, _context: &mut Context) -> failure::Result<Option<String>> {
        let mut reboots = vec![];
        for instance in self.instances.iter() {
            let reboot = Reboot::new(instance.clone());
            reboot.apply()?;
            reboots.push(reboot)
        }
        while reboots.iter().any(|r| !r.is_complete()) {
            thread::sleep(Duration::from_secs(5));
        }
        Ok(None)
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
