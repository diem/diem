// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experiments::ExperimentParam;
use crate::{
    cluster::Cluster,
    effects::{three_region_simulation_effects, Effect},
    experiments::Context,
    experiments::Experiment,
    stats,
    util::unix_timestamp_now,
};
use futures::future::{join_all, BoxFuture, FutureExt};
use std::{
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;

pub struct PerformanceBenchmarkThreeRegionSimulation {
    cluster: Cluster,
}

#[derive(StructOpt, Debug)]
pub struct PerformanceBenchmarkThreeRegionSimulationParams {}

impl ExperimentParam for PerformanceBenchmarkThreeRegionSimulationParams {
    type E = PerformanceBenchmarkThreeRegionSimulation;
    fn build(self, cluster: &Cluster) -> Self::E {
        Self::E {
            cluster: cluster.clone(),
        }
    }
}

impl Experiment for PerformanceBenchmarkThreeRegionSimulation {
    fn run<'a>(
        &'a mut self,
        context: &'a mut Context,
    ) -> BoxFuture<'a, anyhow::Result<Option<String>>> {
        async move {
            let (us, euro) = self.cluster.split_n_validators_random(80);
            let (us_west, us_east) = us.split_n_validators_random(40);
            let network_effects = three_region_simulation_effects(
                (
                    us_west.validator_instances().clone(),
                    us_east.validator_instances().clone(),
                    euro.validator_instances().clone(),
                ),
                (
                    Duration::from_millis(60), // us_east<->eu one way delay
                    Duration::from_millis(95), // us_west<->eu one way delay
                    Duration::from_millis(40), // us_west<->us_east one way delay
                ),
            );
            join_all(network_effects.iter().map(|e| e.activate())).await;
            let window = Duration::from_secs(240);
            context
                .tx_emitter
                .emit_txn_for(window, self.cluster.validator_instances().clone())
                .await?;
            let buffer = Duration::from_secs(30);
            let end = unix_timestamp_now() - buffer;
            let start = end - window + 2 * buffer;
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
