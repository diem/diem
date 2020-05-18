// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
    stats,
    tx_emitter::EmitJobRequest,
    util::unix_timestamp_now,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::future::try_join_all;
use libra_logger::info;
use std::{
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;

pub struct PerformanceBenchmarkThreeRegionSimulation {
    cluster: Cluster,
}

#[derive(StructOpt, Debug)]
pub struct PerformanceBenchmarkThreeRegionSimulationParams {}

impl ExperimentParam for PerformanceBenchmarkThreeRegionSimulationParams {
    type E = PerformanceBenchmarkThreeRegionSimulation;
    fn build(self, cluster: &Cluster) -> Self::E {
        Self::E {
            cluster: cluster.clone(),
        }
    }
}

#[async_trait]
impl Experiment for PerformanceBenchmarkThreeRegionSimulation {
    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let num_nodes = self.cluster.validator_instances().len();
        let split_country_num = ((num_nodes as f64) * 0.8) as usize;
        let split_region_num = split_country_num / 2;
        let (us, euro) = self.cluster.split_n_validators_random(split_country_num);
        let (us_west, us_east) = us.split_n_validators_random(split_region_num);
        three_region_simulation_effects_k8s(
            (
                us_west.validator_instances().to_vec(),
                us_east.validator_instances().to_vec(),
                euro.validator_instances().to_vec(),
            ),
            (
                Duration::from_millis(60), // us_east<->eu one way delay
                Duration::from_millis(95), // us_west<->eu one way delay
                Duration::from_millis(40), // us_west<->us_east one way delay
            ),
            context.cluster_swarm,
        )
        .await?;

        let window = Duration::from_secs(240);
        let emit_job_request = if context.emit_to_validator {
            EmitJobRequest::for_instances(
                context.cluster.validator_instances().to_vec(),
                context.global_emit_job_request,
            )
        } else {
            EmitJobRequest::for_instances(
                context.cluster.fullnode_instances().to_vec(),
                context.global_emit_job_request,
            )
        };
        let stats = context
            .tx_emitter
            .emit_txn_for(window, emit_job_request)
            .await?;
        let buffer = Duration::from_secs(30);
        let end = unix_timestamp_now() - buffer;
        let start = end - window + 2 * buffer;
        let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
        let avg_latency_client = stats.latency as u64 / stats.committed as u64;
        info!(
            "Tx status from client side: txn {}, avg latency {}",
            stats.committed as u64, avg_latency_client
        );
        context.cluster_swarm.remove_all_network_effects().await?;
        context.report.report_metric(&self, "avg_tps", avg_tps);
        context
            .report
            .report_metric(&self, "avg_latency", avg_latency);
        context.report.report_text(format!(
            "{} : {:.0} TPS, {:.1} ms prometheus side latency, {:.1} ms client side latency",
            self, avg_tps, avg_latency, avg_latency_client
        ));
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(420)
    }
}

impl Display for PerformanceBenchmarkThreeRegionSimulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "3 Region Simulation")
    }
}

async fn add_network_delay_k8s(
    instance: Instance,
    configuration: Vec<(Vec<Instance>, Duration)>,
    cluster_swarm: &ClusterSwarmKube,
) -> Result<()> {
    let mut command = "".to_string();
    command += "tc qdisc delete dev eth0 root; ";
    // Create a HTB https://linux.die.net/man/8/tc-htb
    command += "tc qdisc add dev eth0 root handle 1: htb; ";
    for i in 0..configuration.len() {
        // Create a class within the HTB https://linux.die.net/man/8/tc
        command += format!(
            "tc class add dev eth0 parent 1: classid 1:{} htb rate 1tbit; ",
            i + 1
        )
        .as_str();
    }
    for (i, config) in configuration.iter().enumerate() {
        // Create u32 filters so that all the target instances are classified as class 1:(i+1)
        // http://man7.org/linux/man-pages/man8/tc-u32.8.html
        for target_instance in &config.0 {
            command += format!("tc filter add dev eth0 parent 1: protocol ip prio 1 u32 flowid 1:{} match ip dst {}; ", i+1, target_instance.ip()).as_str();
        }
    }
    for (i, config) in configuration.iter().enumerate() {
        // Use netem to delay packets to this class
        command += format!(
            "tc qdisc add dev eth0 parent 1:{} handle {}0: netem delay {}ms; ",
            i + 1,
            i + 1,
            config.1.as_millis(),
        )
        .as_str();
    }

    cluster_swarm
        .run(
            &instance,
            "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
            command,
            "add-network-delay",
        )
        .await
}

async fn three_region_simulation_effects_k8s(
    regions: (Vec<Instance>, Vec<Instance>, Vec<Instance>),
    delays_bw_regions: (Duration, Duration, Duration),
    cluster_swarm: &ClusterSwarmKube,
) -> Result<()> {
    let mut futures = vec![];
    for instance in &regions.0 {
        let configuration = vec![
            (regions.1.clone(), delays_bw_regions.2),
            (regions.2.clone(), delays_bw_regions.1),
        ];
        futures.push(add_network_delay_k8s(
            instance.clone(),
            configuration,
            cluster_swarm,
        ));
    }
    for instance in &regions.1 {
        let configuration = vec![
            (regions.0.clone(), delays_bw_regions.2),
            (regions.2.clone(), delays_bw_regions.0),
        ];
        futures.push(add_network_delay_k8s(
            instance.clone(),
            configuration,
            cluster_swarm,
        ));
    }
    for instance in &regions.2 {
        let configuration = vec![
            (regions.1.clone(), delays_bw_regions.0),
            (regions.0.clone(), delays_bw_regions.1),
        ];
        futures.push(add_network_delay_k8s(
            instance.clone(),
            configuration,
            cluster_swarm,
        ));
    }
    try_join_all(futures).await.map(|_| ())
}
