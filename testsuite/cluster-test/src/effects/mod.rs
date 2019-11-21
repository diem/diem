// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod network_delay;
mod packet_loss;
mod reboot;
mod remove_network_effects;
mod stop_container;

use crate::thread_pool_executor::ThreadPoolExecutor;
use failure;
pub use network_delay::three_region_simulation_effects;
pub use network_delay::NetworkDelay;
pub use packet_loss::PacketLoss;
pub use reboot::Reboot;
pub use remove_network_effects::RemoveNetworkEffects;
use slog_scope::info;
use std::fmt::Display;
pub use stop_container::StopContainer;
pub trait Action: Display + Send {
    fn apply(&self) -> failure::Result<()>;
    fn is_complete(&self) -> bool;
}

pub trait Effect: Display + Send {
    fn activate(&self) -> failure::Result<()>;
    fn deactivate(&self) -> failure::Result<()>;
}

pub fn activate_all<T: Effect>(thread_pool_executor: ThreadPoolExecutor, effects: &mut [T]) {
    let jobs = effects
        .iter_mut()
        .map(|effect| {
            move || {
                if let Err(e) = effect.activate() {
                    info!("Failed to activate {}: {:?}", effect, e);
                }
            }
        })
        .collect();
    thread_pool_executor.execute_jobs(jobs);
}

pub fn deactivate_all<T: Effect>(thread_pool_executor: ThreadPoolExecutor, effects: &mut [T]) {
    let jobs = effects
        .iter_mut()
        .map(|effect| {
            move || {
                if let Err(e) = effect.deactivate() {
                    info!("Failed to deactivate {}: {:?}", effect, e);
                }
            }
        })
        .collect();
    thread_pool_executor.execute_jobs(jobs);
}
