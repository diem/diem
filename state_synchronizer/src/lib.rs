// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![feature(async_await)]
#![recursion_limit = "1024"]
use types::account_address::AccountAddress;

pub use synchronizer::{StateSyncClient, StateSynchronizer};

mod coordinator;
mod counters;
mod synchronizer;

type PeerId = AccountAddress;

#[cfg(test)]
mod tests;
