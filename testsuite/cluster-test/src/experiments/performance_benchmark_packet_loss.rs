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
use futures::join;
use libra_logger::info;
use std::{fmt, time::Duration};
use structopt::StructOpt;
use tokio::time::delay_for;

pub const DEFAULT_BENCH_DURATION_SECS: u64 = 120;

pub struct PerformanceBenchmarkPacketLoss {
    affected_instances: Vec<Instance>,
    percent: f32,
    duration: Duration,
    up_validators: Vec<Instance>,
    up_fullnodes: Vec<Instance>,
}

#[derive(StructOpt, Debug)]
pub struct PerformanceBenchmarkPacketLossParams {
    #[structopt(
        long,
        default_value = "10",
        help = "Percent of instances in which packet loss should be introduced"
    )]
    percent_instances: f32,
    #[structopt(
        long,
        default_value = "10",
        help = "Percent of packet loss for each instance"
    )]
    packet_loss_percent: f32,
    #[structopt(
        long,
        default_value = "60",
        help = "Duration in secs for which packet loss happens"
    )]
    duration_secs: u64,
}

impl ExperimentParam for PerformanceBenchmarkPacketLossParams {
    type E = PerformanceBenchmarkPacketLoss;
    fn build(self, cluster: &Cluster) -> Self::E {
        let total_instances = cluster.validator_instances().len();
        let packet_loss_num_instances: usize = std::cmp::min(
            ((self.percent_instances / 100.0) * total_instances as f32).ceil() as usize,
            total_instances,
        );
//        let (test_cluster, _) = cluster.split_n_validators_random(packet_loss_num_instances);
        let (test_cluster, _) = cluster.split_n_fullnodes_random(packet_loss_num_instances);
        Self::E {
            up_validators: cluster.validator_instances().to_vec(),
            up_fullnodes: cluster.fullnode_instances().to_vec(),
            affected_instances: test_cluster.into_fullnode_instances(),
            percent: self.packet_loss_percent,
            duration: Duration::from_secs(self.duration_secs),
        }
    }
}

#[async_trait]
impl Experiment for PerformanceBenchmarkPacketLoss {
    async fn run(&mut self, context: &mut Context<'_>) -> Result<()> {
        // emit txn
        let buffer = Duration::from_secs(60);
        let window = Duration::from_secs(DEFAULT_BENCH_DURATION_SECS) + buffer * 2;
        let emit_job_request = if context.emit_to_validator {
            EmitJobRequest::for_instances(
                self.up_validators.clone(),
                context.global_emit_job_request,
            )
        } else {
            EmitJobRequest::for_instances(
                self.up_fullnodes.clone(),
                context.global_emit_job_request,
            )
        };
        let emit_txn = context.tx_emitter.emit_txn_for(window, emit_job_request);

        // packet loss
        let packet_loss = self.apply_packet_loss(context.cluster_swarm.clone());

        let (stats, _) = join!(emit_txn, packet_loss);

        // print stats
        let stats = stats?;
        let submitted_txn = stats.submitted;
        let expired_txn = stats.expired;
        let end = unix_timestamp_now() - buffer;
        let start = end - window + 2 * buffer;
        let (avg_tps, avg_latency) = stats::txn_stats(&context.prometheus, start, end)?;
        let avg_txns_per_block = stats::avg_txns_per_block(&context.prometheus, start, end)?;
        info!(
            "Link to dashboard : {}",
            context.prometheus.link_to_dashboard(start, end)
        );
        context
            .report
            .report_metric(&self, "submitted_txn", submitted_txn as f64);
        context
            .report
            .report_metric(&self, "expired_txn", expired_txn as f64);
        context
            .report
            .report_metric(&self, "avg_txns_per_block", avg_txns_per_block as f64);
        context.report.report_metric(&self, "avg_tps", avg_tps);
        context
            .report
            .report_metric(&self, "avg_latency", avg_latency);
        info!("avg_txns_per_block: {}", avg_txns_per_block);
        let expired_text = if expired_txn == 0 {
            "no expired txns".to_string()
        } else {
            format!("(!) expired {} out of {} txns", expired_txn, submitted_txn)
        };
        context.report.report_text(format!(
            "{} : {:.0} TPS, {:.1} ms latency, {}",
            self, avg_tps, avg_latency, expired_text
        ));
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(600)
    }
}

impl PerformanceBenchmarkPacketLoss {
    async fn apply_packet_loss(&mut self, cluster_swarm: ClusterSwarmKube) -> Result<()> {
        for instance in self.affected_instances.iter() {
            add_packet_delay(instance.clone(), self.percent, &cluster_swarm).await?;
        }
        delay_for(self.duration).await;
        cluster_swarm.remove_all_network_effects().await?;
        Ok(())
    }
}

async fn add_packet_delay(
    instance: Instance,
    percent: f32,
    cluster_swarm: &ClusterSwarmKube,
) -> Result<()> {
    let command = format!(
        "tc qdisc delete dev eth0 root; tc qdisc add dev eth0 root netem loss {:.*}%",
        2, percent
    );
    cluster_swarm
        .run(
            &instance,
            "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
            command,
            "add-packet-loss",
        )
        .await
}

impl fmt::Display for PerformanceBenchmarkPacketLoss {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Packet Loss Performance Benchmark {:.*}% [",
            2, self.percent
        )?;
        for instance in self.affected_instances.iter() {
            write!(f, "{}, ", instance)?;
        }
        write!(f, "]")
    }
}
