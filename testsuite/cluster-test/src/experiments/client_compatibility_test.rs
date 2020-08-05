// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
};
use async_trait::async_trait;
use libra_logger::prelude::*;
use std::{collections::HashSet, fmt, time::Duration};
use structopt::StructOpt;
use tokio::time;

#[derive(StructOpt, Debug)]
pub struct ClientCompatiblityTestParams {}

pub struct ClientCompatibilityTest {}

impl ExperimentParam for ClientCompatiblityTestParams {
    type E = ClientCompatibilityTest;
    fn build(self, _cluster: &Cluster) -> Self::E {
        Self::E {}
    }
}

/// TODO(rustielin): this is currently a dummy experiment that tests spawn/kill_job
#[async_trait]
impl Experiment for ClientCompatibilityTest {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let test_image = format!(
            "853397791086.dkr.ecr.us-west-2.amazonaws.com/libra_faucet:{}",
            "master"
        );

        let test_host_node = &context.cluster.fullnode_instances()[0];
        let cmd = "tail -f /dev/null";
        let job_full_name = test_host_node
            .spawn_job(&test_image, &cmd, "run-forever")
            .await?;

        // do some other jobs
        info!("Wait for the other jobs to finish");
        time::delay_for(Duration::from_secs(20)).await;

        context
            .cluster_builder
            .cluster_swarm
            .kill_job(&job_full_name)
            .await?;

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }
}

impl fmt::Display for ClientCompatibilityTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client compatibility test")
    }
}
