// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Consensus for the Libra Core blockchain
//!
//! Encapsulates public consensus traits and any implementations of those traits.
//! Currently, the only consensus protocol supported is LibraBFT (based on
//! [HotStuff](https://arxiv.org/pdf/1803.05069.pdf)).

#![cfg_attr(not(feature = "fuzzing"), deny(missing_docs))]
#![cfg_attr(feature = "fuzzing", allow(dead_code))]
#![recursion_limit = "512"]

#[macro_use]
extern crate prometheus;

mod chained_bft;

mod util;

#[cfg(feature = "fuzzing")]
pub use chained_bft::event_processor_fuzzing;

/// Defines the public consensus provider traits to implement for
/// use in the Libra Core blockchain.
pub mod consensus_provider;

mod counters;

mod state_computer;
mod state_replication;
mod txn_manager;
