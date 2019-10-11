// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![recursion_limit = "1024"]
use libra_types::{account_address::AccountAddress, crypto_proxies::LedgerInfoWithSignatures};

pub use synchronizer::{StateSyncClient, StateSynchronizer};

mod coordinator;
mod counters;
mod executor_proxy;
mod peer_manager;
mod synchronizer;

type PeerId = AccountAddress;
type LedgerInfo = LedgerInfoWithSignatures;

#[cfg(test)]
mod tests;
