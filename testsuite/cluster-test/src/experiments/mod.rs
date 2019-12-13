// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod multi_region_network_simulation;
mod packet_loss_random_validators;
mod performance_benchmark_nodes_down;
mod performance_benchmark_three_region_simulation;
mod reboot_random_validator;
mod recovery_time;

use std::time::Duration;
use std::{collections::HashSet, fmt::Display};

pub use multi_region_network_simulation::{MultiRegionSimulation, MultiRegionSimulationParams};
pub use packet_loss_random_validators::{
    PacketLossRandomValidators, PacketLossRandomValidatorsParams,
};
pub use performance_benchmark_nodes_down::{
    PerformanceBenchmarkNodesDown, PerformanceBenchmarkNodesDownParams,
};
pub use performance_benchmark_three_region_simulation::{
    PerformanceBenchmarkThreeRegionSimulation, PerformanceBenchmarkThreeRegionSimulationParams,
};
pub use reboot_random_validator::{RebootRandomValidators, RebootRandomValidatorsParams};
pub use recovery_time::{RecoveryTime, RecoveryTimeParams};

use crate::cluster::Cluster;
use crate::prometheus::Prometheus;
use crate::tx_emitter::TxEmitter;
use futures::future::BoxFuture;
use std::collections::HashMap;
use structopt::{clap::AppSettings, StructOpt};

pub trait Experiment: Display + Send {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn run<'a>(
        &'a mut self,
        context: &'a mut Context,
    ) -> BoxFuture<'a, anyhow::Result<Option<String>>>;
    fn deadline(&self) -> Duration;
}

pub trait ExperimentParam {
    type E: Experiment;
    fn build(self, cluster: &Cluster) -> Self::E;
}

pub struct Context {
    tx_emitter: TxEmitter,
    prometheus: Prometheus,
    cluster: Cluster,
}

impl Context {
    pub fn new(tx_emitter: TxEmitter, prometheus: Prometheus, cluster: Cluster) -> Self {
        Context {
            tx_emitter,
            prometheus,
            cluster,
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
        "multi_region_simulation",
        f::<MultiRegionSimulationParams>(),
    );
    known_experiments.insert(
        "packet_loss_random_validators",
        f::<PacketLossRandomValidatorsParams>(),
    );
    known_experiments.insert(
        "performance_benchmark_nodes_down",
        f::<PerformanceBenchmarkNodesDownParams>(),
    );
    known_experiments.insert(
        "performance_benchmark_three_region_simulation",
        f::<PerformanceBenchmarkThreeRegionSimulationParams>(),
    );
    known_experiments.insert(
        "reboot_random_validators",
        f::<RebootRandomValidatorsParams>(),
    );

    let builder = known_experiments.get(name).expect("Experiment not found");
    builder(args, cluster)
}
