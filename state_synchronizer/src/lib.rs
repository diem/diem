// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![feature(async_await)]
#![recursion_limit = "1024"]
use crypto::ed25519::*;
use types::{account_address::AccountAddress, ledger_info::LedgerInfoWithSignatures};

pub use synchronizer::{StateSyncClient, StateSynchronizer};

mod coordinator;
mod counters;
mod executor_proxy;
mod synchronizer;

type PeerId = AccountAddress;
type LedgerInfo = LedgerInfoWithSignatures<Ed25519Signature>;

#[cfg(test)]
mod tests;
