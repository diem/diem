// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Consensus for the Libra Core blockchain
//!
//! The consensus protocol implemented is LibraBFT (based on
//! [HotStuff](https://arxiv.org/pdf/1803.05069.pdf)).

#![cfg_attr(not(feature = "fuzzing"), deny(missing_docs))]
#![cfg_attr(feature = "fuzzing", allow(dead_code))]
#![recursion_limit = "512"]

mod block_storage;
mod consensusdb;
mod counters;
mod epoch_manager;
mod event_processor;
mod liveness;
mod network;
#[cfg(test)]
mod network_tests;
mod persistent_liveness_storage;
mod state_computer;
mod state_replication;
#[cfg(any(test, feature = "fuzzing"))]
mod test_utils;
mod txn_manager;
mod util;

/// LibraBFT implementation
pub mod consensus_provider;
/// LibraNet interface.
pub mod network_interface;

#[cfg(feature = "fuzzing")]
pub use event_processor::event_processor_fuzzing;
