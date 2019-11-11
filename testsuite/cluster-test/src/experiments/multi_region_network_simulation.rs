// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This module provides an experiment which simulates a multi-region environment.
/// It undoes the simulation in the cluster after the given duration
use crate::effects::Effect;
use crate::effects::NetworkDelay;
use crate::prometheus::Prometheus;
use crate::tx_emitter::{EmitJobRequest, EmitThreadParams, TxEmitter};
use crate::util::unix_timestamp_now;
use crate::{cluster::Cluster, experiments::Experiment, thread_pool_executor::ThreadPoolExecutor};
use failure;
use slog_scope::{info, warn};
use std::{collections::HashSet, fmt, thread, time::Duration};

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
    splits: Vec<usize>,
    cross_region_latencies: Vec<u64>,
    exp_duration_per_config: Duration,
    thread_pool_executor: ThreadPoolExecutor,
    cluster: Cluster,
    prometheus: Prometheus,
}

impl MultiRegionSimulation {
    pub fn new(
        splits: Vec<usize>,
        cross_region_latencies: Vec<u64>,
        exp_duration_per_config: Duration,
        cluster: Cluster,
        thread_pool_executor: ThreadPoolExecutor,
        prometheus: Prometheus,
    ) -> Self {
        Self {
            splits,
            cross_region_latencies,
            exp_duration_per_config,
            thread_pool_executor,
            cluster,
            prometheus,
        }
    }

    fn single_run(&self, count: usize, cross_region_delay: Duration) -> failure::Result<Metrics> {
        let (cluster1, cluster2) = self.cluster.split_n_random(count);
        let (region1, region2) = (cluster1.into_instances(), cluster2.into_instances());
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
                    Some(larger_region.clone()),
                    cross_region_delay,
                )
            })
            .collect();
        // Add network delay commands only to the instances of the smaller
        // region because we have to run fewer ssh commands.
        let start_effects: Vec<_> = network_delays_effects
            .iter()
            .map(|effect| {
                move || {
                    effect.activate().unwrap();
                }
            })
            .collect();
        self.thread_pool_executor.execute_jobs(start_effects);

        thread::sleep(self.exp_duration_per_config);
        let metrics = self.get_metrics(count, cross_region_delay.as_millis() as u64);
        let stop_effects: Vec<_> = network_delays_effects
            .iter()
            .map(|effect| {
                move || {
                    effect.deactivate().unwrap();
                }
            })
            .collect();
        self.thread_pool_executor.execute_jobs(stop_effects);
        Ok(metrics)
    }

    fn get_metrics(&self, split_size: usize, cross_region_latency: u64) -> Metrics {
        let mut metrics: Metrics = Default::default();
        metrics.split_size = split_size;
        metrics.cross_region_latency = cross_region_latency;
        let step = 10;
        let end = unix_timestamp_now();
        // Measure metrics 30s after the experiment started to ignore initial warmup metrics
        let start = end - self.exp_duration_per_config + Duration::from_secs(30);
        metrics.txn_per_sec = self
            .prometheus
            .query_range_avg(
                "irate(consensus_gauge{op='last_committed_version'}[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok();
        metrics.blocks_per_sec = self
            .prometheus
            .query_range_avg(
                "irate(consensus_gauge{op='last_committed_round'}[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok();
        metrics.avg_vote_time = self
            .prometheus
            .query_range_avg(
                "irate(consensus_duration_sum{op='creation_to_receival_s'}[1m])/irate(consensus_duration_count{op='creation_to_receival_s'}[1m])".to_string(),
                &start,
                &end,
                step,
            )
            .map_err(|e| warn!("{}", e))
            .ok().map(|x| x * 1000f64);
        metrics.avg_qc_time = self
            .prometheus
            .query_range_avg(
                "irate(consensus_duration_sum{op='creation_to_qc_s'}[1m])/irate(consensus_duration_count{op='creation_to_qc_s'}[1m])".to_string(),
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

impl Experiment for MultiRegionSimulation {
    fn affected_validators(&self) -> HashSet<String> {
        let mut r = HashSet::new();
        for instance in self.cluster.clone().into_instances() {
            r.insert(instance.short_hash().clone());
        }
        r
    }

    fn run(&self) -> failure::Result<()> {
        let mut emitter = TxEmitter::new(&self.cluster);
        let mut results = vec![];
        for split in &self.splits {
            for cross_region_latency in &self.cross_region_latencies {
                let job = emitter
                    .start_job(EmitJobRequest {
                        instances: self.cluster.instances().clone(),
                        accounts_per_client: 10,
                        thread_params: EmitThreadParams::default(),
                    })
                    .expect("Failed to start emit job");
                // Wait for minting to complete and transactions to start
                thread::sleep(Duration::from_secs(30));
                info!(
                    "Running multi region simulation: split: {}, cross region latency: {}ms",
                    split, cross_region_latency
                );
                if let Ok(metrics) = self
                    .single_run(*split, Duration::from_millis(*cross_region_latency))
                    .map_err(|e| warn!("{}", e))
                {
                    info!("metrics for this run: {:?}", metrics);
                    results.push(metrics);
                }
                emitter.stop_job(job);
                // Sleep for a minute before different experiments
                thread::sleep(Duration::from_secs(60));
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
            self.splits, self.cross_region_latencies
        )
    }
}
