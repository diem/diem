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
use std::{collections::HashSet, fmt, time::Duration};
use structopt::StructOpt;
use tokio::time;

#[derive(StructOpt, Debug)]
pub struct ClientCompatiblityTestParams {
    #[structopt(long, help = "Image tag of old client to test")]
    pub old_image_tag: String,
}

pub struct ClientCompatibilityTest {
    old_image_tag: String,
    faucet_node: Instance,
    cli_node: Instance,
}

impl ExperimentParam for ClientCompatiblityTestParams {
    type E = ClientCompatibilityTest;
    fn build(self, cluster: &Cluster) -> Self::E {
        let (test_nodes, _) = cluster.split_n_fullnodes_random(2);
        let mut test_nodes = test_nodes.into_fullnode_instances();
        let faucet_node = test_nodes.pop().expect("Requires at least one faucet node");
        let cli_node = test_nodes.pop().expect("Requires at least one test node");
        Self::E {
            old_image_tag: self.old_image_tag,
            faucet_node,
            cli_node,
        }
    }
}

#[async_trait]
impl Experiment for ClientCompatibilityTest {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
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
            "/opt/libra/bin/config-builder faucet -o /opt/libra/etc --chain-id {chain_id} -s {seed} -n {num_validators}; echo $?; cat /opt/libra/etc/waypoint.txt",
            chain_id=ChainId::test(),
            seed=CFG_SEED,
            num_validators=num_validators
        );
        let env_cmd = format!(
            "CFG_CHAIN_ID={chain_id} AC_HOST={ac_host} AC_PORT={ac_port}",
            chain_id = ChainId::test(),
            ac_host = self.faucet_node.ip(),
            ac_port = self.faucet_node.ac_port()
        );
        let run_cmd = format!("gunicorn --bind 0.0.0.0:{faucet_port} --access-logfile - --error-logfile - --log-level debug --pythonpath /opt/libra/bin server", faucet_port=faucet_port);
        // run config builder, set ac and cfg envs, run faucet
        let full_faucet_cmd = format!(
            "{config_cmd}; {env_cmd} {run_cmd}",
            config_cmd = config_cmd,
            env_cmd = env_cmd,
            run_cmd = run_cmd
        );
        let msg = format!("1. Starting faucet on node {}", self.faucet_node);
        info!("{}", msg);
        context.report.report_text(msg);
        let faucet_job_name = self
            .faucet_node
            .spawn_job(&test_image, &full_faucet_cmd, "run-faucet")
            .await
            .map_err(|err| anyhow::format_err!("Failed to spawn faucet job: {}", err))?;
        info!(
            "Job {} started for node {}:{} faucet command: {}",
            faucet_job_name,
            self.faucet_node,
            self.faucet_node.peer_name(),
            full_faucet_cmd
        );

        // wait for faucet to finish starting
        info!("Waiting for faucet job to spin up completely");
        time::delay_for(Duration::from_secs(20)).await;

        // submit a mint request from the test node to the test host (faucet)
        let run_cli_cmd = format!(
            "/opt/libra/bin/cli --url {fn_url} --chain-id {chain_id} -f http://{faucet_host}:{faucet_port} --waypoint $(cat /opt/libra/etc/waypoint.txt)",
            fn_url = self.cli_node.json_rpc_url(),
            chain_id = ChainId::test(),
            faucet_host = self.faucet_node.ip(),
            faucet_port = faucet_port
        );
        let mut build_cli_cmd = String::new();
        let cli_cmd_file = "/opt/libra/etc/cmds.txt";
        let cmds = include_str!("client_compatibility_cmds.txt");
        for cmd in cmds.split('\n') {
            build_cli_cmd.push_str(&format!(
                "echo {cmd} >> {cmd_file};",
                cmd = cmd,
                cmd_file = cli_cmd_file
            ));
        }
        // config builder, build cli cmd on a single line to file, pipe the file input to cli
        let full_cli_cmd = format!(
            "{config_cmd}; {build_cli_cmd} {run_cli_cmd} < {cli_cmd_file} && echo SUCCESS",
            config_cmd = config_cmd,
            build_cli_cmd = build_cli_cmd,
            run_cli_cmd = run_cli_cmd,
            cli_cmd_file = cli_cmd_file
        );
        let msg = format!("2. Running CLI mint from node {}", self.cli_node);
        info!("{}", msg);
        context.report.report_text(msg);
        info!(
            "Job starting for node {}:{} CLI command: {}",
            self.cli_node,
            self.cli_node.peer_name(),
            full_cli_cmd
        );
        // run CLI commands as blocking
        self.cli_node
            .cmd(&test_image, &full_cli_cmd, "run-cli-commands")
            .await
            .map_err(|err| anyhow::format_err!("Failed to run CLI: {}", err))?;

        // verify that CLI actually ran
        let msg = format!("3. CLI success from node {}", self.cli_node);
        info!("{}", msg);
        context.report.report_text(msg);

        // kill faucet if it is still running
        context
            .cluster_builder
            .cluster_swarm
            .kill_job(&faucet_job_name)
            .await
            .map_err(|err| anyhow::format_err!("Failed to kill faucet: {}", err))?;

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }
}

impl fmt::Display for ClientCompatibilityTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Client compatibility test {}, faucet {}, CLI on {}",
            self.old_image_tag, self.faucet_node, self.cli_node
        )
    }
}
