// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![recursion_limit = "1024"]

use libra_types::{
    account_address::AccountAddress, epoch_info::EpochInfo, ledger_info::LedgerInfoWithSignatures,
    transaction::Version, validator_verifier::ValidatorVerifier,
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
///
/// While `highest_local_li` can be used for helping the others (corresponding to the highest
/// version we have a proof for), `synced_trees` is used for retrieving missing chunks
/// for the local storage.
#[derive(Clone)]
pub struct SynchronizerState {
    pub highest_local_li: LedgerInfoWithSignatures,
    // Corresponds to the current epoch if the highest local LI is in the middle of the epoch,
    // or the next epoch if the highest local LI is the final LI in the current epoch.
    pub trusted_epoch: EpochInfo,

    pub highest_sync_version: Version,
}

impl SynchronizerState {
    pub fn new(
        highest_local_li: LedgerInfoWithSignatures,
        highest_sync_version: Version,
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
            highest_sync_version,
            trusted_epoch,
        }
    }

    /// The highest available version in the local storage (even if it's not covered by the LI).
    pub fn highest_version_in_local_storage(&self) -> u64 {
        self.highest_sync_version
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
