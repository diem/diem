// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, time::Duration};

use rand::Rng;

use anyhow::{bail, Result};

use crate::{
    cluster::Cluster,
    cluster_swarm::ClusterSwarm,
    effects::{Action, Reboot},
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
};
use async_trait::async_trait;
use futures::future::{join_all, try_join_all};
use libra_logger::warn;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RebootRandomValidatorsParams {
    #[structopt(long, default_value = "10", help = "Number of nodes to reboot")]
    pub count: usize,
}

pub struct RebootRandomValidators {
    instances: Vec<Instance>,
}

impl ExperimentParam for RebootRandomValidatorsParams {
    type E = RebootRandomValidators;
    fn build(self, cluster: &Cluster) -> Self::E {
        if self.count > cluster.validator_instances().len() {
            panic!(
                "Can not reboot {} validators in cluster with {} instances",
                self.count,
                cluster.validator_instances().len()
            );
        }
        let mut instances = Vec::with_capacity(self.count);
        let mut all_instances = cluster.validator_instances().to_vec();
        let mut rnd = rand::thread_rng();
        for _i in 0..self.count {
            let instance = all_instances.remove(rnd.gen_range(0, all_instances.len()));
            instances.push(instance);
        }
        Self::E { instances }
    }
}

#[async_trait]
impl Experiment for RebootRandomValidators {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.instances)
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        if let Some(cluster_swarm) = context.cluster_swarm {
            let instance_configs = instance::instance_configs(&self.instances)?;
            let futures: Vec<_> = instance_configs
                .clone()
                .into_iter()
                .map(|ic| cluster_swarm.delete_node(ic.clone()))
                .collect();
            try_join_all(futures).await?;
            let futures: Vec<_> = instance_configs
                .into_iter()
                .map(|ic| cluster_swarm.upsert_node(ic.clone(), false))
                .collect();
            try_join_all(futures).await?;
        } else {
            let futures = self.instances.iter().map(|instance| async move {
                let reboot = Reboot::new(instance.clone());
                reboot
                    .apply()
                    .await
                    .map_err(|e| warn!("Failed to reboot {}: {}", instance, e))
            });
            let results = join_all(futures).await;
            if results.iter().any(Result::is_err) {
                bail!("Failed to reboot one of nodes");
            }
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
