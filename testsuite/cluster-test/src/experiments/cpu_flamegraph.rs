// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
};
use anyhow::{anyhow, format_err, Result};
use async_trait::async_trait;
use diem_sdk::transaction_builder::TransactionFactory;
use forge::TxnEmitter;
use futures::{future::FutureExt, join};
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use std::{
    collections::HashSet,
    env,
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
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );
        let buffer = Duration::from_secs(60);
        let tx_emitter_duration = 2 * buffer + Duration::from_secs(self.duration_secs as u64);
        let emit_job_request = crate::util::emit_job_request_for_instances(
            context.cluster.validator_instances().to_vec(),
            context.global_emit_job_request,
            0,
            0,
        );
        let emit_future = txn_emitter
            .emit_txn_for(tx_emitter_duration, emit_job_request)
            .boxed();
        let run_id = env::var("RUN_ID")
            .map_err(|e| anyhow!("RUN_ID could not be read from the environment, Error:{}", e))?;
        let filename = "diem-node-perf.svg";
        let command = generate_perf_flamegraph_command(filename, &run_id, self.duration_secs);
        let flame_graph = self.perf_instance.util_cmd(command, "generate-flamegraph");
        let flame_graph_future = tokio::time::sleep(buffer)
            .then(|_| async move { flame_graph.await })
            .boxed();
        let (emit_result, flame_graph_result) = join!(emit_future, flame_graph_future);
        emit_result.map_err(|e| format_err!("Emiting tx failed: {:?}", e))?;
        flame_graph_result.map_err(|e| format_err!("Failed to generate flamegraph: {:?}", e))?;
        context.report.report_text(format!(
            "perf flamegraph : https://toro-cluster-test-flamegraphs.s3-us-west-2.amazonaws.com/flamegraphs/{}/{}",
            run_id,
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

fn generate_perf_flamegraph_command(filename: &str, run_id: &str, duration_secs: usize) -> String {
    format!(
        r#"
        set -xe;
        rm -rf /tmp/perf-data;
        mkdir /tmp/perf-data;
        cd /tmp/perf-data;
        perf record -F 99 -p $(ps aux | grep diem-node | grep -v grep | awk '{{print $2}}') --output=perf.data --call-graph dwarf -- sleep {duration_secs};
        perf script --input=perf.data | /usr/local/etc/FlameGraph/stackcollapse-perf.pl > out.perf-folded;
        /usr/local/etc/FlameGraph/flamegraph.pl out.perf-folded > {filename};
        aws s3 cp {filename} s3://toro-cluster-test-flamegraphs/flamegraphs/{run_id}/{filename};"#,
        duration_secs = duration_secs,
        filename = filename,
        run_id = run_id,
    )
}
