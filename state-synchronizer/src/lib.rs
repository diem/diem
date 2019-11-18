// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![recursion_limit = "1024"]

#[macro_use]
extern crate prometheus;

use libra_types::{account_address::AccountAddress, crypto_proxies::LedgerInfoWithSignatures};

use libra_crypto::HashValue;
use libra_types::block_info::BlockInfo;
use libra_types::ledger_info::LedgerInfo;
use std::collections::BTreeMap;
pub use synchronizer::{StateSyncClient, StateSynchronizer};

mod coordinator;
mod counters;
mod executor_proxy;
mod peer_manager;
mod synchronizer;

type PeerId = AccountAddress;

#[derive(Clone, Debug, Eq, PartialEq)]
/// The state distinguishes between the following fields:
/// * highest_local_li is keeping the latest certified ledger info
/// * highest_committed_version is keeping the latest version in the transaction accumulator in case
/// the accumulator is ahead of the LedgerInfo.
///
/// While `highest_local_li` can be used for helping the others (corresponding to the highest
/// version we have a proof for), `highest_synced_version` is used for retrieving missing chunks
/// for the local storage.
pub struct SynchronizerState {
    pub highest_local_li: LedgerInfoWithSignatures,
    pub highest_synced_version: u64,
}

impl SynchronizerState {
    pub fn zero_state() -> Self {
        let highest_local_li = LedgerInfoWithSignatures::new(
            LedgerInfo::new(BlockInfo::empty(), HashValue::zero()),
            BTreeMap::new(),
        );
        Self {
            highest_local_li,
            highest_synced_version: 0,
        }
    }

    /// The highest available version in the local storage (even if it's not covered by the LI).
    pub fn highest_version_in_local_storage(&self) -> u64 {
        std::cmp::max(
            self.highest_local_li.ledger_info().version(),
            self.highest_synced_version,
        )
    }
}

#[cfg(test)]
mod tests;
