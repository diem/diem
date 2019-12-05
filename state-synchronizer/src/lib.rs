// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![recursion_limit = "1024"]

#[macro_use]
extern crate prometheus;

use libra_types::{account_address::AccountAddress, crypto_proxies::LedgerInfoWithSignatures};

use libra_types::crypto_proxies::ValidatorVerifier;

pub use synchronizer::{StateSyncClient, StateSynchronizer};

mod chunk_request;
mod chunk_response;
mod coordinator;
mod counters;
mod executor_proxy;
mod peer_manager;
mod synchronizer;

type PeerId = AccountAddress;

#[derive(Clone)]
/// EpochInfo represents the highest trusted validator set: it is updated every time we're crossing
/// the epoch boundary.
pub struct EpochInfo {
    pub epoch: u64,
    pub verifier: ValidatorVerifier,
}

#[derive(Clone)]
/// The state distinguishes between the following fields:
/// * highest_local_li is keeping the latest certified ledger info
/// * highest_synced_version is keeping the latest version in the transaction accumulator in case
/// the accumulator is ahead of the LedgerInfo.
///
/// While `highest_local_li` can be used for helping the others (corresponding to the highest
/// version we have a proof for), `highest_synced_version` is used for retrieving missing chunks
/// for the local storage.
pub struct SynchronizerState {
    pub highest_local_li: LedgerInfoWithSignatures,
    pub highest_synced_version: u64,
    // Corresponds to the current epoch if the highest local LI is in the middle of the epoch,
    // or the next epoch if the highest local LI is the final LI in the current epoch.
    pub trusted_epoch: EpochInfo,
}

impl SynchronizerState {
    pub fn new(
        highest_local_li: LedgerInfoWithSignatures,
        highest_synced_version: u64,
        current_verifier: ValidatorVerifier,
    ) -> Self {
        let current_epoch = highest_local_li.ledger_info().epoch();
        let trusted_epoch = match highest_local_li.ledger_info().next_validator_set() {
            Some(validator_set) => EpochInfo {
                epoch: current_epoch + 1,
                verifier: validator_set.into(),
            },
            None => EpochInfo {
                epoch: current_epoch,
                verifier: current_verifier,
            },
        };
        SynchronizerState {
            highest_local_li,
            highest_synced_version,
            trusted_epoch,
        }
    }

    /// The highest available version in the local storage (even if it's not covered by the LI).
    pub fn highest_version_in_local_storage(&self) -> u64 {
        self.highest_synced_version
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
