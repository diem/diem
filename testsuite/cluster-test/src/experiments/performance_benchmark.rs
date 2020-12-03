// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
    stats::PrometheusRangeView,
    tx_emitter::{EmitJobRequest, TxStats},
    util::human_readable_bytes_per_sec,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use diem_infallible::duration_since_epoch;
use diem_logger::{info, warn};
use diem_trace::{
    trace::{find_peer_with_stage, random_node, trace_node},
    DiemTraceClient,
};
use futures::{future::try_join_all, join, FutureExt};
use rand::{rngs::ThreadRng, seq::SliceRandom};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;
use tokio::task::JoinHandle;

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
    #[structopt(long, help = "Whether benchmark should trace only one diem node")]
    pub trace_single: bool,
    #[structopt(
        long,
        help = "Whether benchmark should perform trace from elastic search logs"
    )]
    pub use_logs_for_trace: bool,
    #[structopt(
    long,
    default_value = Box::leak(format!("{}", DEFAULT_BENCH_DURATION).into_boxed_str()),
    help = "Duration of an experiment in seconds"
    )]
    pub duration: u64,
    #[structopt(long, help = "Set fixed tps during perf experiment")]
    pub tps: Option<u64>,
    #[structopt(
        long,
        help = "Whether benchmark should pick one node to run DB backup."
    )]
    pub backup: bool,
    #[structopt(long, default_value = "0", help = "Set gas price in tx")]
    pub gas_price: u64,
    #[structopt(long, help = "Set periodic stat aggregator step")]
    pub periodic_stats: Option<u64>,
}

pub struct PerformanceBenchmark {
    down_validators: Vec<Instance>,
    up_validators: Vec<Instance>,
    up_fullnodes: Vec<Instance>,
    percent_nodes_down: usize,
    duration: Duration,
    trace: bool,
    trace_single: bool,
    tps: Option<u64>,
    use_logs_for_trace: bool,
    backup: bool,
    gas_price: u64,
    periodic_stats: Option<u64>,
}

pub const DEFAULT_BENCH_DURATION: u64 = 120;

impl PerformanceBenchmarkParams {
    pub fn new_nodes_down(percent_nodes_down: usize) -> Self {
        Self {
            percent_nodes_down,
            duration: DEFAULT_BENCH_DURATION,
            trace: false,
            trace_single: false,
            tps: None,
            use_logs_for_trace: false,
            backup: false,
            gas_price: 0,
            periodic_stats: None,
        }
    }

    pub fn new_fixed_tps(percent_nodes_down: usize, fixed_tps: u64) -> Self {
        Self {
            percent_nodes_down,
            duration: DEFAULT_BENCH_DURATION,
            trace: false,
            trace_single: false,
            tps: Some(fixed_tps),
            use_logs_for_trace: false,
            backup: false,
            gas_price: 0,
            periodic_stats: None,
        }
    }

    pub fn non_zero_gas_price(percent_nodes_down: usize, gas_price: u64) -> Self {
        Self {
            percent_nodes_down,
            duration: DEFAULT_BENCH_DURATION,
            trace: false,
            trace_single: false,
            tps: None,
            use_logs_for_trace: false,
            backup: false,
            gas_price,
            periodic_stats: None,
        }
    }

    pub fn enable_db_backup(mut self) -> Self {
        self.backup = true;
        self
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
            trace_single: self.trace_single,
            tps: self.tps,
            use_logs_for_trace: self.use_logs_for_trace,
            backup: self.backup,
            gas_price: self.gas_price,
            periodic_stats: self.periodic_stats,
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

        let backup = self.maybe_start_backup()?;
        let buffer = Duration::from_secs(60);
        let window = self.duration + buffer * 2;
        let instances = if context.emit_to_validator {
            self.up_validators.clone()
        } else {
            self.up_fullnodes.clone()
        };
        let emit_job_request = match self.tps {
            Some(tps) => EmitJobRequest::fixed_tps(instances, tps, self.gas_price),
            None => EmitJobRequest::for_instances(
                instances,
                context.global_emit_job_request,
                self.gas_price,
            ),
        };
        let emit_txn = match self.periodic_stats {
            Some(interval) => context
                .tx_emitter
                .emit_txn_for_with_stats(window, emit_job_request, interval)
                .boxed(),
            None => context
                .tx_emitter
                .emit_txn_for(window, emit_job_request)
                .boxed(),
        };

        let start = chrono::Utc::now();
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
        let (stats, mut trace) = join!(emit_txn, capture_trace);

        // Trace
        let trace_log = self.use_logs_for_trace;
        if trace_log {
            let start = start + chrono::Duration::seconds(60);
            let diem_trace_client = DiemTraceClient::new("elasticsearch-master", 9200);
            trace = match diem_trace_client
                .get_diem_trace(start, chrono::Duration::seconds(5))
                .await
            {
                Ok(trace) => Some(trace),
                Err(err) => {
                    info!("Failed to capture traces from elastic search {}", err);
                    None
                }
            };
        }
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
                random_node(&events[..], "json-rpc::submit", "txn::").expect("No trace node found");
            info!("Tracing {}", node);
            if self.trace_single {
                let filter_peer =
                    find_peer_with_stage(&events[..], &node, "diem_vm::execute_block_impl")
                        .expect("Can not find peer with diem_vm::execute_block_impl")
                        .to_string();
                events = events
                    .into_iter()
                    .filter(|node| {
                        node.json
                            .as_object()
                            .unwrap()
                            .get("peer")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            == filter_peer
                    })
                    .collect();
            }
            trace_node(&events[..], &node);
        }

        // Report
        self.report(context, buffer, window, stats?).await?;

        // Clean up
        drop(backup);
        let futures: Vec<_> = self.down_validators.iter().map(|ic| ic.start()).collect();
        try_join_all(futures).await?;

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(900) + self.duration
    }
}

impl PerformanceBenchmark {
    fn maybe_start_backup(&self) -> Result<Option<JoinHandle<()>>> {
        if !self.backup {
            return Ok(None);
        }

        let mut rng = ThreadRng::default();
        let validator = self
            .up_validators
            .choose(&mut rng)
            .ok_or_else(|| anyhow!("No up validator."))?
            .clone();

        const COMMAND: &str = "/opt/diem/bin/db-backup coordinator run \
            --transaction-batch-size 20000 \
            --state-snapshot-interval 1 \
            local-fs --dir $(mktemp -d -t diem_backup_XXXXXXXX);";

        Ok(Some(tokio::spawn(async move {
            validator.exec(COMMAND, true).await.unwrap_or_else(|e| {
                let err_msg = e.to_string();
                if err_msg.ends_with("exit code Some(137)") {
                    info!("db-backup killed.");
                } else {
                    warn!("db-backup failed: {}", err_msg);
                }
            })
        })))
    }

    async fn report(
        &mut self,
        context: &mut Context<'_>,
        buffer: Duration,
        window: Duration,
        stats: TxStats,
    ) -> Result<()> {
        let end = duration_since_epoch() - buffer;
        let start = end - window + 2 * buffer;
        info!(
            "Link to dashboard : {}",
            context.prometheus.link_to_dashboard(start, end)
        );

        let pv = PrometheusRangeView::new(&context.prometheus, start, end);

        // Transaction stats
        if let Some(avg_txns_per_block) = pv.avg_txns_per_block() {
            context
                .report
                .report_metric(&self, "avg_txns_per_block", avg_txns_per_block);
        }
        let additional = if self.backup {
            // Backup throughput
            let bytes_per_sec = pv.avg_backup_bytes_per_second().unwrap_or(-1.0);
            context
                .report
                .report_metric(&self, "avg_backup_bytes_per_second", bytes_per_sec);
            format!(" backup: {},", human_readable_bytes_per_sec(bytes_per_sec))
        } else {
            "".to_string()
        };
        context
            .report
            .report_txn_stats(self.to_string(), stats, window, &additional);
        Ok(())
    }
}

impl Display for PerformanceBenchmark {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(tps) = self.tps {
            write!(f, "fixed tps {}", tps)?;
        } else if self.percent_nodes_down == 0 {
            write!(f, "all up")?;
        } else {
            write!(f, "{}% down", self.percent_nodes_down)?;
        }
        if self.gas_price != 0 {
            write!(f, ", gas price {}", self.gas_price)?;
        }
        Ok(())
    }
}
