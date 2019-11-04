// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This module provides an experiment which simulates a multi-region environment.
/// It undoes the simulation in the cluster after the given duration
use crate::effects::NetworkDelay;
use crate::{
    cluster::Cluster,
    effects::{Action, RemoveNetworkEffects},
    experiments::Experiment,
    instance::Instance,
    thread_pool_executor::ThreadPoolExecutor,
};
use failure;
use std::{collections::HashSet, fmt, thread, time::Duration};

pub struct MultiRegionSimulation {
    // Instances in region1
    region1: Vec<Instance>,
    // Instances in region2
    region2: Vec<Instance>,
    cross_region_delay: Duration,
    experiment_duration: Duration,
    thread_pool_executor: ThreadPoolExecutor,
}

impl MultiRegionSimulation {
    pub fn new(
        count: usize,
        cross_region_delay: Duration,
        experiment_duration: Duration,
        cluster: &Cluster,
        thread_pool_executor: ThreadPoolExecutor,
    ) -> Self {
        let (region1, region2) = cluster.split_n_random(count);
        Self {
            region1: region1.into_instances(),
            region2: region2.into_instances(),
            cross_region_delay,
            experiment_duration,
            thread_pool_executor,
        }
    }
}

impl Experiment for MultiRegionSimulation {
    fn affected_validators(&self) -> HashSet<String> {
        let mut r = HashSet::new();
        for instance in self.region1.iter().chain(self.region2.iter()) {
            r.insert(instance.short_hash().clone());
        }
        r
    }

    fn run(&self) -> failure::Result<()> {
        let (smaller_region, larger_region);
        if self.region1.len() < self.region2.len() {
            smaller_region = &self.region1;
            larger_region = &self.region2;
        } else {
            smaller_region = &self.region2;
            larger_region = &self.region1;
        }
        // Add network delay commands only to the instances of the smaller
        // region because we have to run fewer ssh commands.
        let jobs: Vec<_> = smaller_region
            .iter()
            .map(|instance| {
                let instance = instance.clone();
                let larger_region = larger_region.clone();
                let cross_region_delay = self.cross_region_delay;
                move || {
                    let network_delay = NetworkDelay::new(
                        instance.clone(),
                        Some(larger_region.clone()),
                        cross_region_delay,
                    );
                    network_delay.apply()
                }
            })
            .collect();
        self.thread_pool_executor.execute_jobs(jobs);

        thread::sleep(self.experiment_duration);
        for instance in smaller_region {
            let remove_network_effects = RemoveNetworkEffects::new(instance.clone());
            remove_network_effects.apply()?;
        }
        Ok(())
    }
}

impl fmt::Display for MultiRegionSimulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Multi Region Simulation with a split of  {}/{}",
            self.region1.len(),
            self.region2.len()
        )
    }
}
