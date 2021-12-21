// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, time::Duration};

use rand::seq::SliceRandom;

use crate::{
    cluster::Cluster,
    effects::{self, stop_validator::StopValidator},
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
};
use async_trait::async_trait;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RebootRandomValidatorsParams {
    #[structopt(
        long,
        default_value = "10",
        help = "Number of validator nodes to reboot"
    )]
    count: usize,
    #[structopt(long, default_value = "0", help = "Number of lsr nodes to reboot")]
    lsr_count: usize,
}

impl RebootRandomValidatorsParams {
    pub fn new(validator_count: usize, lsr_count: usize) -> Self {
        Self {
            count: validator_count,
            lsr_count,
        }
    }
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

        if self.lsr_count > cluster.lsr_instances().len() {
            panic!(
                "Can not reboot {} lsrs in cluster with {} instances",
                self.count,
                cluster.lsr_instances().len()
            );
        }

        let mut rnd = rand::thread_rng();
        let mut instances = Vec::with_capacity(self.count + self.lsr_count);
        instances.append(
            &mut cluster
                .validator_instances()
                .choose_multiple(&mut rnd, self.count)
                .cloned()
                .collect(),
        );
        instances.append(
            &mut cluster
                .lsr_instances()
                .choose_multiple(&mut rnd, self.lsr_count)
                .cloned()
                .collect(),
        );

        Self::E { instances }
    }
}

#[async_trait]
impl Experiment for RebootRandomValidators {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.instances)
    }

    async fn run(&mut self, _context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut effects: Vec<_> = self
            .instances
            .clone()
            .into_iter()
            .map(StopValidator::new)
            .collect();
        effects::activate_all(&mut effects).await?;
        effects::deactivate_all(&mut effects).await?;
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
