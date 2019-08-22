// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Consensus for the Libra Core blockchain
//!
//! Encapsulates public consensus traits and any implementations of those traits.
//! Currently, the only consensus protocol supported is LibraBFT (based on
//! [HotStuff](https://arxiv.org/pdf/1803.05069.pdf)).

#![deny(missing_docs)]
#![feature(async_await)]
#![recursion_limit = "512"]
extern crate failure;

#[allow(unused_imports)]
#[macro_use]
extern crate debug_interface;

mod chained_bft;
mod util;

/// Defines the public consensus provider traits to implement for
/// use in the Libra Core blockchain.
pub mod consensus_provider;

mod counters;

mod state_computer;
mod state_replication;
mod txn_manager;
