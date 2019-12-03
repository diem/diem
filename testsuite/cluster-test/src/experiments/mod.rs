// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod multi_region_network_simulation;
mod packet_loss_random_validators;
mod performance_benchmark_nodes_down;
mod performance_benchmark_three_region_simulation;
mod reboot_random_validator;

use std::time::Duration;
use std::{collections::HashSet, fmt::Display};

pub use multi_region_network_simulation::{MultiRegionSimulation, MultiRegionSimulationParams};
pub use packet_loss_random_validators::{
    PacketLossRandomValidators, PacketLossRandomValidatorsParams,
};
pub use performance_benchmark_nodes_down::{
    PerformanceBenchmarkNodesDown, PerformanceBenchmarkNodesDownParams,
};
pub use performance_benchmark_three_region_simulation::PerformanceBenchmarkThreeRegionSimulation;
pub use reboot_random_validator::{RebootRandomValidators, RebootRandomValidatorsParams};

use crate::cluster::Cluster;
use crate::prometheus::Prometheus;
use crate::tx_emitter::TxEmitter;
use futures::future::BoxFuture;
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

/// Given an experiment name and its flags, it constructs an instance of that experiment
/// and returns it as a `Box<dyn Experiment>`
pub fn get_experiment(name: &str, args: &[String], cluster: &Cluster) -> Box<dyn Experiment> {
    match name {
        "multi_region_simulation" => Box::new(MultiRegionSimulation::new(
            MultiRegionSimulationParams::from_clap(
                &MultiRegionSimulationParams::clap()
                    .global_setting(AppSettings::NoBinaryName)
                    .get_matches_from(args),
            ),
        )),
        "packet_loss_random_validators" => Box::new(PacketLossRandomValidators::new(
            PacketLossRandomValidatorsParams::from_clap(
                &PacketLossRandomValidatorsParams::clap()
                    .global_setting(AppSettings::NoBinaryName)
                    .get_matches_from(args),
            ),
            cluster,
        )),
        "performance_benchmark_nodes_down" => Box::new(PerformanceBenchmarkNodesDown::new(
            PerformanceBenchmarkNodesDownParams::from_clap(
                &PerformanceBenchmarkNodesDownParams::clap()
                    .global_setting(AppSettings::NoBinaryName)
                    .get_matches_from(args),
            ),
            cluster,
        )),
        "performance_benchmark_three_region_simulation" => {
            Box::new(PerformanceBenchmarkThreeRegionSimulation::new(cluster))
        }
        "reboot_random_validators" => Box::new(RebootRandomValidators::new(
            RebootRandomValidatorsParams::from_clap(
                &RebootRandomValidatorsParams::clap()
                    .global_setting(AppSettings::NoBinaryName)
                    .get_matches_from(args),
            ),
            cluster,
        )),
        name => panic!("Invalid experiment name: {}", name),
    }
}
