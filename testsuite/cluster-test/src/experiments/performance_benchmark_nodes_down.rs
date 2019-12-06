// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experiments::ExperimentParam;
use crate::{
    cluster::Cluster,
    effects::{Effect, StopContainer},
    experiments::Context,
    experiments::Experiment,
    instance,
    instance::Instance,
    stats,
    util::unix_timestamp_now,
};
use futures::future::{join_all, BoxFuture, FutureExt};
use std::{
    collections::HashSet,
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct PerformanceBenchmarkNodesDownParams {
    #[structopt(
        long,
        default_value = "10",
        help = "Number of nodes which should be down"
    )]
    pub num_nodes_down: usize,
}

pub struct PerformanceBenchmarkNodesDown {
    down_instances: Vec<Instance>,
    up_instances: Vec<Instance>,
    num_nodes_down: usize,
}

impl ExperimentParam for PerformanceBenchmarkNodesDownParams {
    type E = PerformanceBenchmarkNodesDown;
    fn build(self, cluster: &Cluster) -> Self::E {
        let (down_instances, up_instances) = cluster.split_n_random(self.num_nodes_down);
        Self::E {
            down_instances: down_instances.into_instances(),
            up_instances: up_instances.into_instances(),
            num_nodes_down: self.num_nodes_down,
        }
    }
}

impl Experiment for PerformanceBenchmarkNodesDown {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.down_instances)
    }

    fn run<'a>(
        &'a mut self,
        context: &'a mut Context,
    ) -> BoxFuture<'a, anyhow::Result<Option<String>>> {
        async move {
            let stop_effects: Vec<_> = self
                .down_instances
                .clone()
                .into_iter()
                .map(StopContainer::new)
                .collect();
            let futures = stop_effects.iter().map(|e| e.activate());
            join_all(futures).await;
            let window = Duration::from_secs(180);
            context
                .tx_emitter
                .emit_txn_for(window + Duration::from_secs(60), self.up_instances.clone())?;
            let end = unix_timestamp_now();
            let start = end - window;
            let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
            let futures = stop_effects.iter().map(|e| e.deactivate());
            join_all(futures).await;
            Ok(Some(format!(
                "{} : {:.0} TPS, {:.1} ms latency",
                self, avg_tps, avg_latency
            )))
        }
            .boxed()
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(480)
    }
}

impl Display for PerformanceBenchmarkNodesDown {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.num_nodes_down == 0 {
            write!(f, "all up")
        } else {
            write!(f, "{}% down", self.num_nodes_down)
        }
    }
}
