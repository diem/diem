// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use failure::_core::fmt::{Error, Formatter};
use failure::_core::time::Duration;

use crate::cluster::Cluster;
use crate::effects::three_region_simulation_effects;
use crate::experiments::Context;
use crate::experiments::Experiment;
use crate::{effects, stats};

use crate::util::unix_timestamp_now;

pub struct PerformanceBenchmarkThreeRegionSimulation {
    cluster: Cluster,
}

impl PerformanceBenchmarkThreeRegionSimulation {
    pub fn new(cluster: Cluster) -> Self {
        Self { cluster }
    }
}

impl Experiment for PerformanceBenchmarkThreeRegionSimulation {
    fn run(&mut self, context: &mut Context) -> failure::Result<Option<String>> {
        let (us, euro) = self.cluster.split_n_random(80);
        let (us_west, us_east) = us.split_n_random(40);
        let mut network_effects = three_region_simulation_effects(
            (
                us_west.instances().clone(),
                us_east.instances().clone(),
                euro.instances().clone(),
            ),
            (
                Duration::from_millis(60), // us_east<->eu one way delay
                Duration::from_millis(95), // us_west<->eu one way delay
                Duration::from_millis(40), // us_west<->us_east one way delay
            ),
        );
        effects::activate_all(context.thread_pool_executor.clone(), &mut network_effects);
        let window = Duration::from_secs(180);
        context.tx_emitter.emit_txn_for(
            window + Duration::from_secs(60),
            self.cluster.instances().clone(),
        )?;
        let end = unix_timestamp_now();
        let start = end - window;
        let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
        effects::deactivate_all(context.thread_pool_executor.clone(), &mut network_effects);
        Ok(Some(format!(
            "{} : {:.0} TPS, {:.1} ms latency",
            self, avg_tps, avg_latency
        )))
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(420)
    }
}

impl Display for PerformanceBenchmarkThreeRegionSimulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "3 Region Simulation")
    }
}
