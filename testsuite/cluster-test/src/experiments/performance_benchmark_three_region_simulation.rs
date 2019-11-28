// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use failure::_core::fmt::{Error, Formatter};
use failure::_core::time::Duration;

use crate::cluster::Cluster;
use crate::effects::{three_region_simulation_effects, Effect};
use crate::experiments::Context;
use crate::experiments::Experiment;
use crate::stats;

use crate::util::unix_timestamp_now;
use futures::future::join_all;
use futures::future::{BoxFuture, FutureExt};

pub struct PerformanceBenchmarkThreeRegionSimulation {
    cluster: Cluster,
}

impl PerformanceBenchmarkThreeRegionSimulation {
    pub fn new(cluster: &Cluster) -> Self {
        Self {
            cluster: cluster.clone(),
        }
    }
}

impl Experiment for PerformanceBenchmarkThreeRegionSimulation {
    fn run<'a>(
        &'a mut self,
        context: &'a mut Context,
    ) -> BoxFuture<'a, failure::Result<Option<String>>> {
        async move {
            let (us, euro) = self.cluster.split_n_random(80);
            let (us_west, us_east) = us.split_n_random(40);
            let network_effects = three_region_simulation_effects(
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
            join_all(network_effects.iter().map(|e| e.activate())).await;
            let window = Duration::from_secs(180);
            context.tx_emitter.emit_txn_for(
                window + Duration::from_secs(60),
                self.cluster.instances().clone(),
            )?;
            let end = unix_timestamp_now();
            let start = end - window;
            let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
            join_all(network_effects.iter().map(|e| e.deactivate())).await;
            Ok(Some(format!(
                "{} : {:.0} TPS, {:.1} ms latency",
                self, avg_tps, avg_latency
            )))
        }
            .boxed()
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
