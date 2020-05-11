// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod cpu_flamegraph;
mod packet_loss_random_validators;
mod performance_benchmark;
mod performance_benchmark_packet_loss;
mod performance_benchmark_three_region_simulation;
mod reboot_random_validator;
mod recovery_time;

use std::{collections::HashSet, fmt::Display, time::Duration};

pub use packet_loss_random_validators::{
    PacketLossRandomValidators, PacketLossRandomValidatorsParams,
};
pub use performance_benchmark::{PerformanceBenchmark, PerformanceBenchmarkParams};
pub use performance_benchmark_packet_loss::{
    PerformanceBenchmarkPacketLoss, PerformanceBenchmarkPacketLossParams,
};
pub use performance_benchmark_three_region_simulation::{
    PerformanceBenchmarkThreeRegionSimulation, PerformanceBenchmarkThreeRegionSimulationParams,
};
pub use reboot_random_validator::{RebootRandomValidators, RebootRandomValidatorsParams};
pub use recovery_time::{RecoveryTime, RecoveryTimeParams};

use crate::{
    cluster::Cluster,
    prometheus::Prometheus,
    report::SuiteReport,
    tx_emitter::{EmitJobRequest, TxEmitter},
};

use crate::{cluster_swarm::cluster_swarm_kube::ClusterSwarmKube, health::TraceTail};
use async_trait::async_trait;
pub use cpu_flamegraph::{CpuFlamegraph, CpuFlamegraphParams};
use std::collections::HashMap;
use structopt::{clap::AppSettings, StructOpt};

#[async_trait]
pub trait Experiment: Display + Send {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }
    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()>;
    fn deadline(&self) -> Duration;
}

pub trait ExperimentParam {
    type E: Experiment;
    fn build(self, cluster: &Cluster) -> Self::E;
}

pub struct Context<'a> {
    pub tx_emitter: &'a mut TxEmitter,
    pub trace_tail: &'a mut TraceTail,
    pub prometheus: &'a Prometheus,
    pub cluster: &'a Cluster,
    pub report: &'a mut SuiteReport,
    pub global_emit_job_request: &'a mut Option<EmitJobRequest>,
    pub emit_to_validator: bool,
    pub cluster_swarm: &'a ClusterSwarmKube,
}

impl<'a> Context<'a> {
    pub fn new(
        tx_emitter: &'a mut TxEmitter,
        trace_tail: &'a mut TraceTail,
        prometheus: &'a Prometheus,
        cluster: &'a Cluster,
        report: &'a mut SuiteReport,
        emit_job_request: &'a mut Option<EmitJobRequest>,
        emit_to_validator: bool,
        cluster_swarm: &'a ClusterSwarmKube,
    ) -> Self {
        Context {
            tx_emitter,
            trace_tail,
            prometheus,
            cluster,
            report,
            global_emit_job_request: emit_job_request,
            emit_to_validator,
            cluster_swarm,
        }
    }
}

fn from_args<P: ExperimentParam>(args: &[String], cluster: &Cluster) -> Box<dyn Experiment>
where
    P: StructOpt + 'static,
{
    let params = P::from_clap(
        &P::clap()
            .global_setting(AppSettings::NoBinaryName)
            .get_matches_from(args),
    );
    Box::new(params.build(cluster))
}

/// Given an experiment name and its flags, it constructs an instance of that experiment
/// and returns it as a `Box<dyn Experiment>`
pub fn get_experiment(name: &str, args: &[String], cluster: &Cluster) -> Box<dyn Experiment> {
    fn f<P: ExperimentParam + StructOpt + 'static>(
    ) -> Box<dyn Fn(&[String], &Cluster) -> Box<dyn Experiment>> {
        Box::new(from_args::<P>)
    }

    let mut known_experiments = HashMap::new();

    known_experiments.insert("recovery_time", f::<RecoveryTimeParams>());
    known_experiments.insert(
        "packet_loss_random_validators",
        f::<PacketLossRandomValidatorsParams>(),
    );
    known_experiments.insert("bench", f::<PerformanceBenchmarkParams>());
    known_experiments.insert(
        "bench_packet_loss",
        f::<PerformanceBenchmarkPacketLossParams>(),
    );
    known_experiments.insert(
        "bench_three_region",
        f::<PerformanceBenchmarkThreeRegionSimulationParams>(),
    );
    known_experiments.insert(
        "reboot_random_validators",
        f::<RebootRandomValidatorsParams>(),
    );
    known_experiments.insert("generate_cpu_flamegraph", f::<CpuFlamegraphParams>());

    let builder = known_experiments.get(name).expect("Experiment not found");
    builder(args, cluster)
}
