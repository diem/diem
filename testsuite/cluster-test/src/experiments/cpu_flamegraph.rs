// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experiments::ExperimentParam;
use crate::{
    aws, cluster::Cluster, experiments::Context, experiments::Experiment, instance,
    instance::Instance,
};

use crate::effects::{Action, GenerateCpuFlamegraph};
use anyhow::{format_err, Result};
use async_trait::async_trait;
use chrono::{Datelike, Timelike, Utc};
use futures::future::FutureExt;
use futures::join;
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

#[async_trait]
impl Experiment for CpuFlamegraph {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&[self.perf_instance.clone()])
    }

    async fn run(&mut self, context: &mut Context<'_>) -> Result<()> {
        let buffer = Duration::from_secs(60);
        let tx_emitter_duration = 2 * buffer + Duration::from_secs(self.duration_secs as u64);
        let emit_future = context
            .tx_emitter
            .emit_txn_for(
                tx_emitter_duration,
                context.cluster.validator_instances().clone(),
            )
            .boxed();
        let flame_graph =
            GenerateCpuFlamegraph::new(self.perf_instance.clone(), self.duration_secs);
        let flame_graph_future = tokio::time::delay_for(buffer)
            .then(|_| flame_graph.apply())
            .boxed();
        let (emit_result, flame_graph_result) = join!(emit_future, flame_graph_future);
        emit_result.map_err(|e| format_err!("Emiting tx failed: {:?}", e))?;
        flame_graph_result.map_err(|e| format_err!("Failed to generate flamegraph: {:?}", e))?;
        let now = Utc::now();
        let filename = format!(
            "{:04}{:02}{:02}-{:02}{:02}{:02}-libra-node-perf.svg",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        aws::upload_to_s3(
            "perf-kernel.svg",
            "toro-cluster-test-flamegraphs",
            filename.as_str(),
        )?;
        context.report.report_text(format!(
            "perf flamegraph : https://s3.console.aws.amazon.com/s3/object/toro-cluster-test-flamegraphs/{}",
            filename
        ));
        Ok(())
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
