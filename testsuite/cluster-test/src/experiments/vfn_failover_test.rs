// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
    tx_emitter::EmitJobRequest,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{join, FutureExt};
use std::{collections::HashSet, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct VfnFailoverParams {
    //    #[structopt(long, default_value = "1", help = "Number of validators to bring down")]
//    pub count: u64,
}

pub struct VfnFailover {}

impl ExperimentParam for VfnFailoverParams {
    type E = VfnFailover;
    fn build(self, _cluster: &Cluster) -> Self::E {
        Self::E {}
    }
}

#[async_trait]
impl Experiment for VfnFailover {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> Result<()> {
        // send txns to all fn
        let instances: Vec<Instance> = context.cluster.fullnode_instances().to_vec();

        let emit_job_request = EmitJobRequest::for_instances(
            instances,
            context.global_emit_job_request,
            0, // gas price
        );

        let window = Duration::from_secs(240);

        let emit_txn = context
            .tx_emitter
            .emit_txn_for(window, emit_job_request)
            .boxed();

        let v_to_fail = context.cluster.random_validator_instance();
        // bring down one validator for one minute
        let restart_validator = async move {
            println!("in job bringing V down and back");
            tokio::time::delay_for(Duration::from_secs(60)).await;
            v_to_fail.stop().await.expect("failed to fail V");
            tokio::time::delay_for(Duration::from_secs(15)).await;
            v_to_fail.start().await.expect("failed to restart down V");
        };

        let (stats, _) = join!(emit_txn, restart_validator);

        context
            .report
            .report_txn_stats(self.to_string(), stats?, window, "");

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(900) + Duration::from_secs(120)
    }
}

impl std::fmt::Display for VfnFailover {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VFN failover")
    }
}
