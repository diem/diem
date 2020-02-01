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
use slog_scope::info;
use std::sync::atomic::Ordering;
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
        default_value = "0",
        help = "Number of nodes which should be down"
    )]
    pub num_nodes_down: usize,
    #[structopt(
        long,
        help = "Whether cluster test should run against validators or full nodes"
    )]
    pub is_fullnode: bool,
}

pub struct PerformanceBenchmarkNodesDown {
    down_instances: Vec<Instance>,
    up_instances: Vec<Instance>,
    num_nodes_down: usize,
}

impl ExperimentParam for PerformanceBenchmarkNodesDownParams {
    type E = PerformanceBenchmarkNodesDown;
    fn build(self, cluster: &Cluster) -> Self::E {
        if self.is_fullnode {
            let (down_instances, up_instances) =
                cluster.split_n_fullnodes_random(self.num_nodes_down);
            Self::E {
                down_instances: down_instances.into_fullnode_instances(),
                up_instances: up_instances.into_fullnode_instances(),
                num_nodes_down: self.num_nodes_down,
            }
        } else {
            let (down_instances, up_instances) =
                cluster.split_n_validators_random(self.num_nodes_down);
            Self::E {
                down_instances: down_instances.into_validator_instances(),
                up_instances: up_instances.into_validator_instances(),
                num_nodes_down: self.num_nodes_down,
            }
        }
    }
}

impl Experiment for PerformanceBenchmarkNodesDown {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.down_instances)
    }

    fn run<'a>(&'a mut self, context: &'a mut Context) -> BoxFuture<'a, anyhow::Result<()>> {
        async move {
            let stop_effects: Vec<_> = self
                .down_instances
                .clone()
                .into_iter()
                .map(StopContainer::new)
                .collect();
            let futures = stop_effects.iter().map(|e| e.activate());
            join_all(futures).await;
            let window = Duration::from_secs(240);
            let stats = context
                .tx_emitter
                .emit_txn_for(window, self.up_instances.clone())
                .await?;
            let buffer = Duration::from_secs(30);
            let end = unix_timestamp_now() - buffer;
            let start = end - window + 2 * buffer;
            let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
            info!(
                "Link to dashboard : {}",
                context.prometheus.link_to_dashboard(start, end)
            );
            let futures = stop_effects.iter().map(|e| e.deactivate());
            join_all(futures).await;
            let submitted_txn = stats.submitted.load(Ordering::Relaxed);
            let expired_txn = stats.expired.load(Ordering::Relaxed);
            context
                .report
                .report_metric(&self, "submitted_txn", submitted_txn as f64);
            context
                .report
                .report_metric(&self, "expired_txn", expired_txn as f64);
            context.report.report_metric(&self, "avg_tps", avg_tps);
            context
                .report
                .report_metric(&self, "avg_latency", avg_latency);
            let expired_text = if expired_txn == 0 {
                "no expired txns".to_string()
            } else {
                format!("(!) expired {} out of {} txns", expired_txn, submitted_txn)
            };
            context.report.report_text(format!(
                "{} : {:.0} TPS, {:.1} ms latency, {}",
                self, avg_tps, avg_latency, expired_text
            ));
            Ok(())
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
