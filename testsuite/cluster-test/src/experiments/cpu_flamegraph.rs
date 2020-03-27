// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashSet,
    fmt::{Display, Error, Formatter},
    time::Duration,
};

use anyhow::{format_err, Result};

use futures::{future::FutureExt, join};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use structopt::StructOpt;

use async_trait::async_trait;

use crate::{
    aws,
    cluster::Cluster,
    cluster_swarm::ClusterSwarm,
    effects::{Action, GenerateCpuFlamegraph},
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
    tx_emitter::EmitJobRequest,
};

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
        let emit_job_request = EmitJobRequest::for_instances(
            context.cluster.validator_instances().to_vec(),
            context.global_emit_job_request,
        );
        let emit_future = context
            .tx_emitter
            .emit_txn_for(tx_emitter_duration, emit_job_request)
            .boxed();
        let filename = format!(
            "{}-libra-node-perf.svg",
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .collect::<String>()
        );
        if let Some(cluster_swarm) = context.cluster_swarm {
            let command = generate_perf_flamegraph_command(&filename, self.duration_secs);
            let flame_graph = cluster_swarm.run(
                &self.perf_instance,
                "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
                command,
                "generate-flamegraph",
            );
            let flame_graph_future = tokio::time::delay_for(buffer)
                .then(|_| async move { flame_graph.await })
                .boxed();
            let (emit_result, flame_graph_result) = join!(emit_future, flame_graph_future);
            emit_result.map_err(|e| format_err!("Emiting tx failed: {:?}", e))?;
            flame_graph_result
                .map_err(|e| format_err!("Failed to generate flamegraph: {:?}", e))?;
        } else {
            let flame_graph =
                GenerateCpuFlamegraph::new(self.perf_instance.clone(), self.duration_secs);
            let flame_graph_future = tokio::time::delay_for(buffer)
                .then(|_| flame_graph.apply())
                .boxed();
            let (emit_result, flame_graph_result) = join!(emit_future, flame_graph_future);
            emit_result.map_err(|e| format_err!("Emiting tx failed: {:?}", e))?;
            flame_graph_result
                .map_err(|e| format_err!("Failed to generate flamegraph: {:?}", e))?;
            aws::upload_to_s3(
                "perf-kernel.svg",
                "toro-cluster-test-flamegraphs",
                filename.as_str(),
                Some("image/svg+xml".to_string()),
            )?;
        }
        context.report.report_text(format!(
            "perf flamegraph : https://toro-cluster-test-flamegraphs.s3-us-west-2.amazonaws.com/{}",
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

fn generate_perf_flamegraph_command(filename: &str, duration_secs: usize) -> String {
    format!(
        r#"
        set -xe;
        rm -rf /tmp/perf-data;
        mkdir /tmp/perf-data;
        cd /tmp/perf-data;
        perf record -F 99 -p $(ps aux | grep libra-node | grep -v grep | awk '{{print $2}}') --output=perf.data --call-graph dwarf -- sleep {duration_secs};
        perf script --input=perf.data | /usr/local/etc/FlameGraph/stackcollapse-perf.pl > out.perf-folded;
        /usr/local/etc/FlameGraph/flamegraph.pl out.perf-folded > {filename};
        aws s3 cp {filename} s3://toro-cluster-test-flamegraphs/{filename};"#,
        duration_secs = duration_secs,
        filename = filename,
    )
}
