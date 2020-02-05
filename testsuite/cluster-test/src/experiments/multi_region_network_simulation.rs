// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use std::{fmt, time::Duration};

use anyhow::Result;
use slog_scope::{info, warn};
use structopt::StructOpt;

use tokio::time;

/// This module provides an experiment which simulates a multi-region environment.
/// It undoes the simulation in the cluster after the given duration
use crate::effects::Effect;
use crate::effects::NetworkDelay;
use crate::experiments::{Context, ExperimentParam};

use crate::cluster::Cluster;
use crate::experiments::Experiment;
use crate::tx_emitter::{EmitJobRequest, TxEmitter};
use crate::util::unix_timestamp_now;
use async_trait::async_trait;
use futures::future::join_all;

#[derive(Default, Debug)]
struct Metrics {
    split_size: usize,
    cross_region_latency: u64,
    txn_per_sec: Option<f64>,
    blocks_per_sec: Option<f64>,
    avg_vote_time: Option<f64>,
    avg_qc_time: Option<f64>,
}

pub struct MultiRegionSimulation {
    params: MultiRegionSimulationParams,
}

#[derive(StructOpt, Debug)]
pub struct MultiRegionSimulationParams {
    #[structopt(
        long,
        default_value = "10",
        help = "Space separated list of various split sizes"
    )]
    splits: Vec<usize>,
    #[structopt(
        long,
        default_value = "50",
        help = "Space separated list of various delays in ms between the two regions"
    )]
    cross_region_latencies: Vec<u64>,
    #[structopt(
        long,
        default_value = "300",
        help = "Duration in secs (per config) for which multi region experiment happens"
    )]
    duration_secs: u64,
}

impl ExperimentParam for MultiRegionSimulationParams {
    type E = MultiRegionSimulation;
    fn build(self, _cluster: &Cluster) -> Self::E {
        Self::E { params: self }
    }
}

impl MultiRegionSimulation {
    async fn single_run<'a>(
        &'a self,
        count: usize,
        cross_region_delay: Duration,
        context: &'a mut Context<'_>,
    ) -> Result<Metrics> {
        let (cluster1, cluster2) = context.cluster.split_n_validators_random(count);
        let (region1, region2) = (
            cluster1.into_validator_instances(),
            cluster2.into_validator_instances(),
        );
        let (smaller_region, larger_region);
        if region1.len() < region2.len() {
            smaller_region = &region1;
            larger_region = &region2;
        } else {
            smaller_region = &region2;
            larger_region = &region1;
        }
        let network_delays_effects: Vec<_> = smaller_region
            .iter()
            .map(|instance| {
                NetworkDelay::new(
                    instance.clone(),
                    vec![(larger_region.clone(), cross_region_delay)],
                )
            })
            .collect();
        // Add network delay commands only to the instances of the smaller
        // region because we have to run fewer ssh commands.
        let start_effects = network_delays_effects
            .iter()
            .map(|effect| effect.activate());
        join_all(start_effects).await;

        time::delay_for(self.exp_duration_per_config()).await;
        let metrics = self.get_metrics(count, cross_region_delay.as_millis() as u64, context);
        let stop_effects = network_delays_effects
            .iter()
            .map(|effect| effect.deactivate());
        join_all(stop_effects).await;
        Ok(metrics)
    }

    fn exp_duration_per_config(&self) -> Duration {
        Duration::from_secs(self.params.duration_secs)
    }

    fn get_metrics(
        &self,
        split_size: usize,
        cross_region_latency: u64,
        context: &mut Context,
    ) -> Metrics {
        let mut metrics: Metrics = Default::default();
        metrics.split_size = split_size;
        metrics.cross_region_latency = cross_region_latency;
        let step = 10;
        let end = unix_timestamp_now();
        // Measure metrics 30s after the experiment started to ignore initial warmup metrics
        let start = end - self.exp_duration_per_config() + Duration::from_secs(30);
        metrics.txn_per_sec = context
            .prometheus
            .query_range_avg(
                "irate(libra_consensus_last_committed_version[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok();
        metrics.blocks_per_sec = context
            .prometheus
            .query_range_avg(
                "irate(libra_consensus_last_committed_round[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok();
        metrics.avg_vote_time = context
            .prometheus
            .query_range_avg(
                "irate(libra_consensus_creation_to_receival_s_sum[1m])/irate(libra_consensus_creation_to_receival_s_count[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok().map(|x| x * 1000f64);
        metrics.avg_qc_time = context
            .prometheus
            .query_range_avg(
                "irate(libra_consensus_creation_to_qc_s_sum[1m])/irate(libra_consensus_creation_to_qc_s_count[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok().map(|x| x * 1000f64);
        metrics
    }
}

fn print_results(metrics: Vec<Metrics>) {
    info!("Results:");
    println!(
        "split_size, cross_region_latency, txn_per_sec, blocks_per_sec, avg_vote_time, avg_qc_time"
    );
    let to_str = |x: &Option<f64>| -> String {
        if let Some(val) = x {
            format!("{:.2}", val)
        } else {
            "".to_string()
        }
    };
    for metric in metrics {
        println!(
            "{}, {}, {}, {}, {}, {}",
            metric.split_size,
            metric.cross_region_latency,
            to_str(&metric.txn_per_sec),
            to_str(&metric.blocks_per_sec),
            to_str(&metric.avg_vote_time),
            to_str(&metric.avg_qc_time),
        );
    }
}

#[async_trait]
impl Experiment for MultiRegionSimulation {
    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut emitter = TxEmitter::new(&context.cluster);
        let mut results = vec![];
        for split in &self.params.splits {
            for cross_region_latency in &self.params.cross_region_latencies {
                let job = emitter
                    .start_job(EmitJobRequest::for_instances(
                        context.cluster.validator_instances().clone(),
                    ))
                    .await
                    .expect("Failed to start emit job");
                // Wait for minting to complete and transactions to start
                time::delay_for(Duration::from_secs(30)).await;
                info!(
                    "Running multi region simulation: split: {}, cross region latency: {}ms",
                    split, cross_region_latency
                );
                if let Ok(metrics) = self
                    .single_run(
                        *split,
                        Duration::from_millis(*cross_region_latency),
                        context,
                    )
                    .await
                    .map_err(|e| warn!("{}", e))
                {
                    info!("metrics for this run: {:?}", metrics);
                    results.push(metrics);
                }
                emitter.stop_job(job);
                // Sleep for a minute before different experiments
                time::delay_for(Duration::from_secs(60)).await;
            }
        }
        print_results(results);
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(24 * 60 * 60)
    }
}

impl fmt::Display for MultiRegionSimulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Multi Region Simulation splits: {:?} cross region latencies: {:?}",
            self.params.splits, self.params.cross_region_latencies
        )
    }
}
