// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experiments::ExperimentParam;
use crate::{
    cluster::Cluster, experiments::Context, experiments::Experiment, instance, instance::Instance,
};
use futures::future::{BoxFuture, FutureExt};
use futures::join;
use crate::effects::{Action, GenerateCpuFlamegraph};
use std::{
    collections::HashSet,
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CpuFlamegraphParams {
    #[structopt(
        long,
        default_value = "60",
        help = "Number of seconds for which perf should be run"
    )]
    pub duration_secs: usize,
}

pub struct CpuFlamegraph {
    duration_secs: usize,
    perf_instance: Instance,
}

impl ExperimentParam for CpuFlamegraphParams {
    type E = CpuFlamegraph;
    fn build(self, cluster: &Cluster) -> Self::E {
        let perf_instance = cluster.random_validator_instance();
        Self::E {
            duration_secs: self.duration_secs,
            perf_instance,
        }
    }
}

impl Experiment for CpuFlamegraph {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&[self.perf_instance.clone()])
    }

    fn run<'a>(
        &'a mut self,
        context: &'a mut Context,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        async move {
            let buffer = Duration::from_secs(60);
            let tx_emitter_duration = 2 * buffer + Duration::from_secs(self.duration_secs as u64);

            let emit_future = context.tx_emitter.emit_txn_for(tx_emitter_duration, context.cluster.validator_instances().clone());
            let flame_graph_future = GenerateCpuFlamegraph::new(self.perf_instance.clone(), self.duration_secs).apply();
            let flame_graph_future = tokio::time::delay_for(buffer).then(|_| flame_graph_future);
            let (emit_result, flame_graph_result) =  join!(emit_future, flame_graph_future);
            emit_result?;
            flame_graph_result?;
            Ok(())
        }
        .boxed()
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(480)
    }
}

impl Display for CpuFlamegraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Generating CpuFlamegraph on {}", self.perf_instance)
    }
}
