// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    cluster_swarm::cluster_swarm_kube::CFG_SEED,
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
};
use async_trait::async_trait;
use libra_logger::prelude::*;
use libra_types::chain_id::ChainId;
use std::{collections::HashSet, fmt, iter::once, time::Duration};
use structopt::StructOpt;
use tokio::time;

#[derive(StructOpt, Debug)]
pub struct ClientCompatiblityTestParams {
    #[structopt(long, help = "Image tag of old client to test")]
    pub old_image_tag: String,
}

pub struct ClientCompatibilityTest {
    old_image_tag: String,
    test_host: Instance,
    test_node: Instance,
}

impl ExperimentParam for ClientCompatiblityTestParams {
    type E = ClientCompatibilityTest;
    fn build(self, cluster: &Cluster) -> Self::E {
        let (test_nodes, _) = cluster.split_n_fullnodes_random(2);
        let mut test_nodes = test_nodes.into_fullnode_instances();
        let test_host = test_nodes.pop().expect("Requires at least one test host");
        let test_node = test_nodes.pop().expect("Requires at least one test node");
        Self::E {
            old_image_tag: self.old_image_tag,
            test_host,
            test_node,
        }
    }
}

#[async_trait]
impl Experiment for ClientCompatibilityTest {
    fn affected_validators(&self) -> HashSet<String> {
        once(self.test_node.peer_name().clone())
            .chain(once(self.test_host.peer_name().clone()))
            .collect()
    }
    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        context.report.report_text(format!(
            "Client compatibility test results for {} ==> {} (PR)",
            self.old_image_tag, context.current_tag
        ));
        let test_image = format!(
            "853397791086.dkr.ecr.us-west-2.amazonaws.com/libra_faucet:{}",
            self.old_image_tag
        );

        // run faucet on the test host
        let faucet_port: &str = "9999";
        let num_validators = context.cluster.validator_instances().len();
        let config_cmd = format!(
            "/opt/libra/bin/config-builder faucet -o /opt/libra/etc --chain-id {} -s {} -n {}; echo $?; cat /opt/libra/etc/waypoint.txt",
            ChainId::test(),
            CFG_SEED,
            num_validators
        );
        let env_cmd = format!(
            "CFG_CHAIN_ID={} AC_HOST={} AC_PORT={}",
            ChainId::test(),
            self.test_host.ip(),
            self.test_host.ac_port()
        );
        let run_cmd = format!("gunicorn --bind 0.0.0.0:{} --access-logfile - --error-logfile - --log-level debug --pythonpath /opt/libra/bin server", faucet_port);
        // run config builder, set ac and cfg envs, run faucet
        let full_faucet_cmd = format!("{}; {} {}", config_cmd, env_cmd, run_cmd);
        let msg = format!("1. Starting faucet on node {}", self.test_host);
        info!("{}", msg);
        context.report.report_text(msg);
        let faucet_job_name = self
            .test_host
            .spawn_job(&test_image, &full_faucet_cmd, "run-faucet")
            .await?;
        info!(
            "Job {} started for node {}:{} faucet command: {}",
            faucet_job_name,
            self.test_host,
            self.test_host.peer_name(),
            full_faucet_cmd
        );

        // wait for faucet to finish starting
        info!("Waiting for faucet job to spin up completely");
        time::delay_for(Duration::from_secs(20)).await;

        // submit a mint request from the test node to the test host (faucet)
        let run_cli_cmd = format!("/opt/libra/bin/cli --url {} --chain-id {} -f http://{}:{} --waypoint $(cat /opt/libra/etc/waypoint.txt.fail)", self.test_node.json_rpc_url(), ChainId::test(), self.test_host.ip(), faucet_port);
        let mut cli_cmd = String::new();
        let cli_cmd_file = "/opt/libra/etc/cmds.txt";
        let cmds = include_str!("client_compatibility_cmds.txt");
        for cmd in cmds.split('\n') {
            cli_cmd.push_str(&format!("echo {} >> {};", cmd, cli_cmd_file));
        }
        // config builder, build cli cmd on a single line to file, pipe the file input to cli
        let full_cli_cmd = format!(
            "{}; {} {} < {} && echo SUCCESS",
            config_cmd, cli_cmd, run_cli_cmd, cli_cmd_file
        );
        let msg = format!("2. Running CLI mint from node {}", self.test_node);
        info!("{}", msg);
        context.report.report_text(msg);
        info!(
            "Job starting for node {}:{} CLI command: {}",
            self.test_node,
            self.test_node.peer_name(),
            full_cli_cmd
        );
        // run CLI commands as blocking
        self.test_node
            .cmd(&test_image, &full_cli_cmd, "run-cli-commands")
            .await?;

        // verify that CLI actually ran
        let msg = format!("3. CLI success from node {}", self.test_node);
        info!("{}", msg);
        context.report.report_text(msg);

        // kill faucet if it is still running
        context
            .cluster_builder
            .cluster_swarm
            .kill_job(&faucet_job_name)
            .await?;

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(16 * 60)
    }
}

impl fmt::Display for ClientCompatibilityTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Client compatibility test {}, faucet {}, CLI on {}",
            self.old_image_tag, self.test_host, self.test_node
        )
    }
}
