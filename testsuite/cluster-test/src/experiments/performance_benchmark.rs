// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
    stats,
    tx_emitter::EmitJobRequest,
    util::unix_timestamp_now,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{future::try_join_all, join};
use libra_logger::{info, warn};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct PerformanceBenchmarkParams {
    #[structopt(
        long,
        default_value = "0",
        help = "Percent of nodes which should be down"
    )]
    pub percent_nodes_down: usize,
    #[structopt(long, help = "Whether benchmark should perform trace")]
    pub trace: bool,
    #[structopt(
    long,
    default_value = Box::leak(format!("{}", DEFAULT_BENCH_DURATION).into_boxed_str()),
    help = "Duration of an experiment in seconds"
    )]
    pub duration: u64,
    #[structopt(long, help = "Set fixed tps during perf experiment")]
    pub tps: Option<u64>,
}

pub struct PerformanceBenchmark {
    down_validators: Vec<Instance>,
    up_validators: Vec<Instance>,
    up_fullnodes: Vec<Instance>,
    percent_nodes_down: usize,
    duration: Duration,
    trace: bool,
    tps: Option<u64>,
}

pub const DEFAULT_BENCH_DURATION: u64 = 120;

impl PerformanceBenchmarkParams {
    pub fn new_nodes_down(percent_nodes_down: usize) -> Self {
        Self {
            percent_nodes_down,
            duration: DEFAULT_BENCH_DURATION,
            trace: false,
            tps: None,
        }
    }

    pub fn new_fixed_tps(percent_nodes_down: usize, fixed_tps: u64) -> Self {
        Self {
            percent_nodes_down,
            duration: DEFAULT_BENCH_DURATION,
            trace: false,
            tps: Some(fixed_tps),
        }
    }
}

impl ExperimentParam for PerformanceBenchmarkParams {
    type E = PerformanceBenchmark;
    fn build(self, cluster: &Cluster) -> Self::E {
        let all_fullnode_instances = cluster.fullnode_instances();
        let num_nodes = cluster.validator_instances().len();
        let nodes_down = (num_nodes * self.percent_nodes_down) / 100;
        let (down, up) = cluster.split_n_validators_random(nodes_down);
        let up_validators = up.into_validator_instances();
        let up_fullnodes: Vec<_> = up_validators
            .iter()
            .filter_map(|val| {
                all_fullnode_instances
                    .iter()
                    .find(|x| val.validator_group() == x.validator_group())
                    .cloned()
            })
            .collect();
        Self::E {
            down_validators: down.into_validator_instances(),
            up_validators,
            up_fullnodes,
            percent_nodes_down: self.percent_nodes_down,
            duration: Duration::from_secs(self.duration),
            trace: self.trace,
            tps: self.tps,
        }
    }
}

#[async_trait]
impl Experiment for PerformanceBenchmark {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.down_validators)
    }

    async fn run(&mut self, context: &mut Context<'_>) -> Result<()> {
        let futures: Vec<_> = self.down_validators.iter().map(Instance::stop).collect();
        try_join_all(futures).await?;
        let buffer = Duration::from_secs(60);
        let window = self.duration + buffer * 2;
        let instances = if context.emit_to_validator {
            self.up_validators.clone()
        } else {
            self.up_fullnodes.clone()
        };
        let emit_job_request = match self.tps {
            Some(tps) => EmitJobRequest::fixed_tps(instances, tps),
            None => EmitJobRequest::for_instances(instances, context.global_emit_job_request),
        };
        let emit_txn = context.tx_emitter.emit_txn_for(window, emit_job_request);
        let trace_tail = &context.trace_tail;
        let trace_delay = buffer;
        let trace = self.trace;
        let capture_trace = async move {
            if trace {
                tokio::time::delay_for(trace_delay).await;
                Some(trace_tail.capture_trace(Duration::from_secs(5)).await)
            } else {
                None
            }
        };
        let (stats, trace) = join!(emit_txn, capture_trace);
        let stats = stats?;
        if let Some(trace) = trace {
            info!("Traced {} events", trace.len());
            let mut events = vec![];
            for (node, mut event) in trace {
                // This could be done more elegantly, but for now this will do
                event
                    .json
                    .as_object_mut()
                    .unwrap()
                    .insert("peer".to_string(), Value::String(node));
                events.push(event);
            }
            events.sort_by_key(|k| k.timestamp);
            let node =
                debug_interface::libra_trace::random_node(&events[..], "json-rpc::submit", "txn::")
                    .expect("No trace node found");
            info!("Tracing {}", node);
            debug_interface::libra_trace::trace_node(&events[..], &node);
        }
        let end = unix_timestamp_now() - buffer;
        let start = end - window + 2 * buffer;
        let avg_txns_per_block = stats::avg_txns_per_block(&context.prometheus, start, end);
        let avg_txns_per_block = avg_txns_per_block
            .map_err(|e| warn!("Failed to query avg_txns_per_block: {}", e))
            .ok();
        let avg_latency_client = stats.latency / stats.committed;
        let p99_latency = stats.latency_buckets.percentile(99, 100);
        let avg_tps = stats.committed / window.as_secs();
        info!(
            "Link to dashboard : {}",
            context.prometheus.link_to_dashboard(start, end)
        );
        info!(
            "Tx status: txn {}, avg latency {}",
            stats.committed as u64, avg_latency_client
        );
        let futures: Vec<_> = self
            .down_validators
            .iter()
            .map(|ic| ic.start(false))
            .collect();
        try_join_all(futures).await?;
        let submitted_txn = stats.submitted;
        let expired_txn = stats.expired;
        context
            .report
            .report_metric(&self, "submitted_txn", submitted_txn as f64);
        context
            .report
            .report_metric(&self, "expired_txn", expired_txn as f64);
        if let Some(avg_txns_per_block) = avg_txns_per_block {
            context
                .report
                .report_metric(&self, "avg_txns_per_block", avg_txns_per_block);
        }
        context
            .report
            .report_metric(&self, "avg_tps", avg_tps as f64);
        context
            .report
            .report_metric(&self, "avg_latency", avg_latency_client as f64);
        context
            .report
            .report_metric(&self, "p99_latency", p99_latency as f64);
        let expired_text = if expired_txn == 0 {
            "no expired txns".to_string()
        } else {
            format!("(!) expired {} out of {} txns", expired_txn, submitted_txn)
        };
        context.report.report_text(format!(
            "{} : {:.0} TPS, {:.1} ms latency, {:.1} ms p99 latency, {}",
            self, avg_tps, avg_latency_client, p99_latency, expired_text
        ));
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(600) + self.duration
    }
}

impl Display for PerformanceBenchmark {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(tps) = self.tps {
            write!(f, "fixed tps {}", tps)
        } else if self.percent_nodes_down == 0 {
            write!(f, "all up")
        } else {
            write!(f, "{}% down", self.percent_nodes_down)
        }
    }
}
