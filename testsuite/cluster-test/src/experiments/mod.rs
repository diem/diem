// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod multi_region_network_simulation;
mod packet_loss_random_validators;
mod reboot_random_validator;

pub use multi_region_network_simulation::{MultiRegionSimulation, MultiRegionSimulationParams};
pub use packet_loss_random_validators::{
    PacketLossRandomValidators, PacketLossRandomValidatorsParams,
};
pub use reboot_random_validator::RebootRandomValidators;
use std::time::Duration;
use std::{collections::HashSet, fmt::Display};

pub trait Experiment: Display + Send {
    fn affected_validators(&self) -> HashSet<String>;
    fn run(&self) -> failure::Result<()>;
    fn deadline(&self) -> Duration;
}
