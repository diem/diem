// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod accurate_measurment;
mod compatibility_test;
mod cpu_flamegraph;
mod load_test;
mod packet_loss_random_validators;
mod performance_benchmark;
mod performance_benchmark_three_region_simulation;
mod reboot_cluster;
mod reboot_random_validators;
mod reconfiguration_test;
mod recovery_time;
mod state_sync_performance;
mod twin_validator;
mod versioning_test;

use crate::{
    cluster::Cluster,
    cluster_builder::{ClusterBuilder, ClusterBuilderParams},
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
    experiments::accurate_measurment::AccurateMeasurementParams,
    prometheus::Prometheus,
    report::SuiteReport,
};
use async_trait::async_trait;
use diem_sdk::types::LocalAccount;
use forge::EmitJobRequest;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    time::Duration,
};
use structopt::{clap::AppSettings, StructOpt};

pub use compatibility_test::{CompatibilityTest, CompatiblityTestParams};
pub use cpu_flamegraph::{CpuFlamegraph, CpuFlamegraphParams};
pub use load_test::LoadTestParams;
pub use packet_loss_random_validators::{
    PacketLossRandomValidators, PacketLossRandomValidatorsParams,
};
pub use performance_benchmark::{PerformanceBenchmark, PerformanceBenchmarkParams};
pub use performance_benchmark_three_region_simulation::{
    PerformanceBenchmarkThreeRegionSimulation, PerformanceBenchmarkThreeRegionSimulationParams,
};
pub use reboot_cluster::{RebootCluster, RebootClusterParams};
pub use reboot_random_validators::{RebootRandomValidators, RebootRandomValidatorsParams};
pub use reconfiguration_test::{Reconfiguration, ReconfigurationParams};
pub use recovery_time::{RecoveryTime, RecoveryTimeParams};
pub use state_sync_performance::{StateSyncPerformance, StateSyncPerformanceParams};
pub use twin_validator::{TwinValidators, TwinValidatorsParams};
pub use versioning_test::{ValidatorVersioning, ValidatorVersioningParams};

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
    pub root_account: &'a mut LocalAccount,
    pub treasury_compliance_account: &'a mut LocalAccount,
    pub designated_dealer_account: &'a mut LocalAccount,
    pub prometheus: &'a Prometheus,
    pub cluster_builder: &'a mut ClusterBuilder,
    pub cluster_builder_params: &'a ClusterBuilderParams,
    pub cluster: &'a Cluster,
    pub report: &'a mut SuiteReport,
    pub global_emit_job_request: &'a mut Option<EmitJobRequest>,
    pub emit_to_validator: bool,
    pub cluster_swarm: &'a dyn ClusterSwarm,
    /// Current docker image tag used by this run
    pub current_tag: &'a str,
}

impl<'a> Context<'a> {
    pub fn new(
        root_account: &'a mut LocalAccount,
        treasury_compliance_account: &'a mut LocalAccount,
        designated_dealer_account: &'a mut LocalAccount,
        prometheus: &'a Prometheus,
        cluster_builder: &'a mut ClusterBuilder,
        cluster_builder_params: &'a ClusterBuilderParams,
        cluster: &'a Cluster,
        report: &'a mut SuiteReport,
        emit_job_request: &'a mut Option<EmitJobRequest>,
        emit_to_validator: bool,
        cluster_swarm: &'a ClusterSwarmKube,
        current_tag: &'a str,
    ) -> Self {
        Context {
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            prometheus,
            cluster_builder,
            cluster_builder_params,
            cluster,
            report,
            global_emit_job_request: emit_job_request,
            emit_to_validator,
            cluster_swarm,
            current_tag,
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
        "bench_three_region",
        f::<PerformanceBenchmarkThreeRegionSimulationParams>(),
    );
    known_experiments.insert(
        "reboot_random_validators",
        f::<RebootRandomValidatorsParams>(),
    );
    known_experiments.insert("twin", f::<TwinValidatorsParams>());
    known_experiments.insert("generate_cpu_flamegraph", f::<CpuFlamegraphParams>());
    known_experiments.insert("versioning_testing", f::<ValidatorVersioningParams>());
    known_experiments.insert("compatibility_test", f::<CompatiblityTestParams>());
    known_experiments.insert("reboot_cluster", f::<RebootClusterParams>());
    known_experiments.insert("reconfiguration", f::<ReconfigurationParams>());
    known_experiments.insert("load_test", f::<LoadTestParams>());
    known_experiments.insert("state_sync_performance", f::<StateSyncPerformanceParams>());
    known_experiments.insert("measure", f::<AccurateMeasurementParams>());

    let builder = known_experiments.get(name).expect("Experiment not found");
    builder(args, cluster)
}
