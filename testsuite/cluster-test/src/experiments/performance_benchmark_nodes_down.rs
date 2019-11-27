// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashSet, fmt::Display};

use failure::_core::fmt::{Error, Formatter};
use failure::_core::time::Duration;

use crate::cluster::Cluster;
use crate::effects::StopContainer;
use crate::experiments::Context;
use crate::experiments::Experiment;
use crate::instance::Instance;
use crate::{effects, instance, stats};

use crate::util::unix_timestamp_now;

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

impl PerformanceBenchmarkNodesDown {
    pub fn new(params: PerformanceBenchmarkNodesDownParams, cluster: &Cluster) -> Self {
        let (down_instances, up_instances) = cluster.split_n_random(params.num_nodes_down);
        Self {
            down_instances: down_instances.into_instances(),
            up_instances: up_instances.into_instances(),
            num_nodes_down: params.num_nodes_down,
        }
    }
}

impl Experiment for PerformanceBenchmarkNodesDown {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.down_instances)
    }

    fn run(&mut self, context: &mut Context) -> failure::Result<Option<String>> {
        let mut stop_effects: Vec<_> = self
            .down_instances
            .clone()
            .into_iter()
            .map(StopContainer::new)
            .collect();
        effects::activate_all(context.thread_pool_executor.clone(), &mut stop_effects);
        let window = Duration::from_secs(180);
        context
            .tx_emitter
            .emit_txn_for(window + Duration::from_secs(60), self.up_instances.clone())?;
        let end = unix_timestamp_now();
        let start = end - window;
        let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
        effects::deactivate_all(context.thread_pool_executor.clone(), &mut stop_effects);
        Ok(Some(format!(
            "{} : {:.0} TPS, {:.1} ms latency",
            self, avg_tps, avg_latency
        )))
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
