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

use async_trait::async_trait;
pub use generate_cpu_flamegraph::GenerateCpuFlamegraph;
pub use network_delay::three_region_simulation_effects;
pub use network_delay::NetworkDelay;
pub use packet_loss::PacketLoss;
pub use reboot::Reboot;
pub use remove_network_effects::RemoveNetworkEffects;
use std::fmt::Display;
pub use stop_container::StopContainer;

#[async_trait]
pub trait Action: Display + Send {
    async fn apply(&self) -> Result<()>;
}

#[async_trait]
pub trait Effect: Display + Send {
    async fn activate(&self) -> Result<()>;
    async fn deactivate(&self) -> Result<()>;
}
