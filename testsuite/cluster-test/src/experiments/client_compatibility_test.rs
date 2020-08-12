// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
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
    py_node: Instance,
}

impl ExperimentParam for ClientCompatiblityTestParams {
    type E = ClientCompatibilityTest;
    fn build(self, cluster: &Cluster) -> Self::E {
        let (test_nodes, _) = cluster.split_n_fullnodes_random(3);
        let mut test_nodes = test_nodes.into_fullnode_instances();
        let faucet_node = test_nodes.pop().expect("Requires at least one faucet node");
        let cli_node = test_nodes
            .pop()
            .expect("Requires at least one CLI test node");
        let py_node = test_nodes
            .pop()
            .expect("Requires at least one python test node");
        Self::E {
            old_image_tag: self.old_image_tag,
            faucet_node,
            cli_node,
            py_node,
        }
    }
}

/// Given a local file, build a shell command that will reconstruct the file at some remote destination
pub fn get_file_build_cmd(src_contents: &str, dst_file_name: &str) -> String {
    let mut build_cmd = String::new();
    let cmd_file = dst_file_name;
    let cmds = src_contents;
    for cmd in cmds.split('\n') {
        // NOTE: some ugly string replacement might be necessary to play nicely when it gets stuffed in k8s job command
        build_cmd.push_str(&format!(
            "echo '{cmd}' >> {cmd_file};",
            cmd = cmd.clone(),
            cmd_file = cmd_file
        ));
    }
    build_cmd
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
        let waypoint = context.cluster.get_waypoint().expect("Missing waypoint");
        let config_cmd = format!(
            "echo {waypoint} > /opt/libra/etc/waypoint.txt",
            waypoint = waypoint
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
        let cli_cmd_file = "/opt/libra/etc/cmds.txt";
        let build_cli_cmd =
            get_file_build_cmd(include_str!("client_compatibility_cmds.txt"), cli_cmd_file);

        // build cli cmd on a single line to file, pipe the file input to cli
        let full_cli_cmd = format!(
            "{build_cli_cmd} {run_cli_cmd} < {cli_cmd_file} && echo SUCCESS",
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

        // construct pylibra commands
        let py_cmd_file = "/opt/libra/etc/cmds.py";
        let build_py_cmd =
            get_file_build_cmd(include_str!("client_compatibility_cmds.py"), py_cmd_file);

        let py_env_cmd = format!(
            "JSON_RPC_URL={fn_url} FAUCET_URL='http://{faucet_host}:{faucet_port}'",
            fn_url = self.py_node.json_rpc_url(),
            faucet_host = self.faucet_node.ip(),
            faucet_port = faucet_port
        );

        // build py cmd on a single line to file, pipe the file input to python
        let full_py_cmd = format!(
            "{build_py_cmd} cat /opt/libra/etc/cmds.py; {py_env_cmd} /usr/bin/python3 < {py_cmd_file} && echo SUCCESS",
            build_py_cmd = build_py_cmd,
            py_env_cmd = py_env_cmd,
            py_cmd_file = py_cmd_file
        );
        let msg = format!("4. Running CLI mint from node {}", self.py_node);
        info!("{}", msg);
        context.report.report_text(msg);
        info!(
            "Job starting for node {}:{} Py command: {}",
            self.py_node,
            self.py_node.peer_name(),
            full_py_cmd
        );
        // run python commands as blocking
        self.py_node
            .cmd(&test_image, &full_py_cmd, "run-py-commands")
            .await
            .map_err(|err| anyhow::format_err!("Failed to run python: {}", err))?;

        // verify that python actually ran
        let msg = format!("5. Pylibra success from node {}", self.py_node);
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
            "Client compatibility test {}, faucet {}, CLI {}, pylibra {}",
            self.old_image_tag, self.faucet_node, self.cli_node, self.py_node
        )
    }
}
