// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Consensus for the Diem Core blockchain
//!
//! The consensus protocol implemented is DiemBFT (based on
//! [HotStuff](https://arxiv.org/pdf/1803.05069.pdf)).

#![cfg_attr(not(feature = "fuzzing"), deny(missing_docs))]
#![cfg_attr(feature = "fuzzing", allow(dead_code))]
#![recursion_limit = "512"]

mod block_storage;
mod consensusdb;
mod counters;
mod epoch_manager;
mod error;
mod experimental;
mod liveness;
mod logging;
mod metrics_safety_rules;
mod network;
#[cfg(test)]
mod network_tests;
mod pending_votes;
mod persistent_liveness_storage;
mod round_manager;
mod state_computer;
mod state_replication;
#[cfg(any(test, feature = "fuzzing"))]
mod test_utils;
#[cfg(test)]
mod twins;
mod txn_manager;
mod util;

/// DiemBFT implementation
pub mod consensus_provider;
/// DiemNet interface.
pub mod network_interface;

#[cfg(feature = "fuzzing")]
pub use round_manager::round_manager_fuzzing;
