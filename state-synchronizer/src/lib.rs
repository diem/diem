// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![recursion_limit = "1024"]

#[macro_use]
extern crate prometheus;

use executor_types::ExecutedTrees;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{EpochInfo, LedgerInfoWithSignatures, ValidatorVerifier},
};
use std::sync::Arc;
pub use synchronizer::{StateSyncClient, StateSynchronizer};

mod chunk_request;
mod chunk_response;
mod coordinator;
mod counters;
mod executor_proxy;
pub mod network;
mod peer_manager;
mod synchronizer;

type PeerId = AccountAddress;

/// The state distinguishes between the following fields:
/// * highest_local_li is keeping the latest certified ledger info
/// * synced_trees is keeping the latest state in the transaction accumulator and state tree.
///
/// While `highest_local_li` can be used for helping the others (corresponding to the highest
/// version we have a proof for), `synced_trees` is used for retrieving missing chunks
/// for the local storage.
#[derive(Clone)]
pub struct SynchronizerState {
    pub highest_local_li: LedgerInfoWithSignatures,
    pub synced_trees: ExecutedTrees,
    // Corresponds to the current epoch if the highest local LI is in the middle of the epoch,
    // or the next epoch if the highest local LI is the final LI in the current epoch.
    pub trusted_epoch: EpochInfo,
}

impl SynchronizerState {
    pub fn new(
        highest_local_li: LedgerInfoWithSignatures,
        synced_trees: ExecutedTrees,
        current_verifier: ValidatorVerifier,
    ) -> Self {
        let current_epoch = highest_local_li.ledger_info().epoch();
        let trusted_epoch = match highest_local_li.ledger_info().next_validator_set() {
            Some(validator_set) => EpochInfo {
                epoch: current_epoch + 1,
                verifier: Arc::new(validator_set.into()),
            },
            None => EpochInfo {
                epoch: current_epoch,
                verifier: Arc::new(current_verifier),
            },
        };
        SynchronizerState {
            highest_local_li,
            synced_trees,
            trusted_epoch,
        }
    }

    pub fn highest_local_timestamp(&self) -> u64 {
        self.highest_local_li.timestamp_usecs()
    }

    /// The highest available version in the local storage (even if it's not covered by the LI).
    pub fn highest_version_in_local_storage(&self) -> u64 {
        self.synced_trees.version().unwrap_or(0)
    }

    pub fn epoch(&self) -> u64 {
        self.trusted_epoch.epoch
    }

    pub fn verifier(&self) -> &ValidatorVerifier {
        &self.trusted_epoch.verifier
    }
}

#[cfg(test)]
mod tests;
