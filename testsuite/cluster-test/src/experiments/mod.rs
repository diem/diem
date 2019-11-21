// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;
use std::{collections::HashSet, fmt::Display};

pub use multi_region_network_simulation::{MultiRegionSimulation, MultiRegionSimulationParams};
pub use packet_loss_random_validators::{
    PacketLossRandomValidators, PacketLossRandomValidatorsParams,
};
pub use performance_benchmark_nodes_down::PerformanceBenchmarkNodesDown;
pub use performance_benchmark_three_region_simulation::PerformanceBenchmarkThreeRegionSimulation;
pub use reboot_random_validator::RebootRandomValidators;

use crate::prometheus::Prometheus;
use crate::thread_pool_executor::ThreadPoolExecutor;
use crate::tx_emitter::TxEmitter;

mod multi_region_network_simulation;
mod packet_loss_random_validators;
mod performance_benchmark_nodes_down;
mod performance_benchmark_three_region_simulation;
mod reboot_random_validator;

pub trait Experiment: Display + Send {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn run(&mut self, context: &mut Context) -> failure::Result<Option<String>>;
    fn deadline(&self) -> Duration;
}

pub struct Context {
    tx_emitter: TxEmitter,
    prometheus: Prometheus,
    thread_pool_executor: ThreadPoolExecutor,
}

impl Context {
    pub fn new(
        tx_emitter: TxEmitter,
        prometheus: Prometheus,
        thread_pool_executor: ThreadPoolExecutor,
    ) -> Self {
        Context {
            tx_emitter,
            prometheus,
            thread_pool_executor,
        }
    }
}
