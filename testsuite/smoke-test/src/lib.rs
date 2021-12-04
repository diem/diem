// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Defines Forge Tests
pub mod event_fetcher;
pub mod fullnode;
pub mod replay_tooling;
pub mod rest_api;
pub mod scripts_and_modules;
pub mod transaction;
pub mod verifying_client;

// Converted to local Forge backend
#[cfg(test)]
mod client;
#[cfg(test)]
mod consensus;
#[cfg(test)]
mod full_nodes;
#[cfg(test)]
mod genesis;
#[cfg(test)]
mod key_manager;
#[cfg(test)]
mod network;
#[cfg(test)]
mod operational_tooling;
#[cfg(test)]
mod release_flow;
#[cfg(test)]
mod state_sync;
#[cfg(test)]
mod state_sync_v2;
#[cfg(test)]
mod storage;

#[cfg(test)]
mod smoke_test_environment;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod workspace_builder;
