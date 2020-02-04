// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod delete_libra_data;
mod generate_cpu_flamegraph;
mod network_delay;
mod packet_loss;
mod reboot;
mod remove_network_effects;
mod stop_container;

use anyhow::Result;
pub use delete_libra_data::DeleteLibraData;
use futures::future::BoxFuture;
pub use generate_cpu_flamegraph::GenerateCpuFlamegraph;
pub use network_delay::three_region_simulation_effects;
pub use network_delay::NetworkDelay;
pub use packet_loss::PacketLoss;
pub use reboot::Reboot;
pub use remove_network_effects::RemoveNetworkEffects;
use std::fmt::Display;
pub use stop_container::StopContainer;

pub trait Action: Display + Send {
    fn apply(&self) -> BoxFuture<Result<()>>;
}

pub trait Effect: Display + Send {
    fn activate(&self) -> BoxFuture<Result<()>>;
    fn deactivate(&self) -> BoxFuture<Result<()>>;
}
