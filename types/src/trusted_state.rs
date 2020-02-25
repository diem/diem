// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    crypto_proxies::{EpochInfo, LedgerInfoWithSignatures, ValidatorSet},
    get_with_proof::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse},
    ledger_info::LedgerInfo,
    transaction::Version,
    validator_change::VerifierType,
    waypoint::Waypoint,
};
use anyhow::{format_err, Context as _, Result};
use std::sync::Arc;

/// `TrustedState` keeps track of our latest trusted state, including the latest
/// verified version and the latest verified validator set.
#[derive(Clone, Debug)]
pub struct TrustedState {
    /// The latest verified version of either a waypoint or a ledger info, either
    /// inside an epoch or the epoch change ledger info. If the TrustedState is
    /// generated from an initial waypoint, the latest_version is coincidentally
    /// the same as the waypoint version.
    latest_version: Version,
    /// The current verifier. If we're starting up fresh, this is probably a
    /// waypoint from our config. Otherwise, this is probably generated from the
    /// validator set in the last known epoch change ledger info.
    verifier: VerifierType,
}

/// `TrustedStateChange` is the result of attempting to ratchet to a new trusted
/// state. In order to reduce redundant error checking, `TrustedStateChange` also
/// contains references to relevant items used to ratchet us.
#[derive(Clone, Debug)]
pub enum TrustedStateChange<'a> {
    /// We have a newer `TrustedState` but it's still in the same epoch, so only
    /// the latest trusted version changed.
    Version {
        new_state: TrustedState,
        latest_li: &'a LedgerInfoWithSignatures,
    },
    /// We have a newer `TrustedState` and there was at least one epoch change,
    /// so we have a newer trusted version and a newer trusted validator set.
    Epoch {
        new_state: TrustedState,
        latest_li: &'a LedgerInfoWithSignatures,
        latest_epoch_change_li: &'a LedgerInfoWithSignatures,
        latest_validator_set: &'a ValidatorSet,
    },
    /// Our current `TrustedState` is more recent.
    Stale,
}

impl TrustedState {
    /// Create an initial trusted state from a waypoint.
    pub fn from_waypoint(waypoint: Waypoint) -> Self {
        Self {
            latest_version: waypoint.version(),
            verifier: VerifierType::Waypoint(waypoint),
        }
    }

    /// Create an initial trusted state that will trust the first genesis
    /// presented to it.
    ///
    /// WARNING: this is obviously unsafe, as a malicious peer could present any
    /// arbitrary genesis and this TrustedState would gladly accept it.
    // TODO(philiphayes/dmitrip): remove this when waypoints are completely
    // integrated with client code.
    #[allow(non_snake_case)]
    pub fn new_trust_any_genesis_WARNING_UNSAFE() -> Self {
        Self {
            latest_version: 0,
            verifier: VerifierType::TrustedVerifier(EpochInfo::empty()),
        }
    }

    /// Create an initial trusted state from an epoch change ledger info.
    pub fn from_epoch_change_ledger_info(epoch_change_li: &LedgerInfo) -> Result<Self> {
        // Generate the EpochInfo from the new validator set.
        let epoch_info = EpochInfo {
            epoch: epoch_change_li.epoch() + 1,
            verifier: Arc::new(
                epoch_change_li
                    .next_validator_set()
                    .ok_or_else(|| {
                        format_err!(
                            "No ValidatorSet in LedgerInfo; it must not be on an epoch boundary"
                        )
                    })?
                    .into(),
            ),
        };
        let latest_version = epoch_change_li.version();
        let verifier = VerifierType::TrustedVerifier(epoch_info);

        Ok(Self {
            latest_version,
            verifier,
        })
    }

    /// Verify and ratchet forward our trusted state from an UpdateToLatestLedger
    /// request/response.
    ///
    /// For example, a client sends an `UpdateToLatestLedgerRequest` to a
    /// FullNode and receive some validator change proof along with a latest
    /// ledger info inside the `UpdateToLatestLedgerResponse`. This function
    /// verifies the response, the change proof, and ratchets the trusted state
    /// version forward if the response successfully moves us into a new epoch
    /// or a new latest ledger info within our current epoch.
    ///
    /// + If there was a validation error, e.g., the epoch change proof was
    /// invalid, we return an `Err`.
    ///
    /// + If the message was well formed but stale (i.e., the returned latest
    /// ledger is behind our trusted version), we return
    /// `Ok(TrustedStateChange::Stale)`.
    ///
    /// + If the response is fresh and there is no epoch change, we just ratchet
    /// our trusted version to the latest ledger info and return
    /// `Ok(TrustedStateChange::Version { .. })`.
    ///
    /// + If there is a new epoch and the server provides a correct proof, we
    /// ratchet our trusted version forward, update our verifier to contain
    /// the new validator set, and return `Ok(TrustedStateChange::Epoch { .. })`.
    pub fn verify_and_ratchet_response<'a>(
        &self,
        req: &UpdateToLatestLedgerRequest,
        res: &'a UpdateToLatestLedgerResponse,
    ) -> Result<TrustedStateChange<'a>> {
        let res_version = res.ledger_info_with_sigs.ledger_info().version();
        if res_version < self.latest_version {
            // The response is stale, so we don't need to update our trusted
            // state.
            return Ok(TrustedStateChange::Stale);
        }

        let maybe_epoch_info = res
            .verify(&self.verifier, req)
            .context("Error verifying UpdateToLatestLedgerResponse")?;

        let trusted_state_change = match maybe_epoch_info {
            // After processing this UpdateToLatestLedgerResponse, we're able to
            // enter a new epoch. Move to the latest validator set and ratchet
            // our version.
            Some(epoch_info) => {
                // ratchet our latest version

                let verifier = VerifierType::TrustedVerifier(epoch_info);
                let latest_li = &res.ledger_info_with_sigs;

                let latest_epoch_change_li = res
                    .validator_change_proof
                    .ledger_info_with_sigs
                    .last()
                    // TODO(philiphayes): Should this panic? We cannot have moved
                    // to a new epoch with an empty ValidatorChangeProof.
                    .ok_or_else(|| format_err!("A valid ValidatorChangeProof cannot be empty"))?;

                let latest_validator_set = latest_epoch_change_li
                    .ledger_info()
                    .next_validator_set()
                    // TODO(philiphayes): Should this panic? We cannot have a
                    // valid ValidatorChangeProof where an epoch change ledger
                    // info does not have a new validator set.
                    .ok_or_else(|| {
                        format_err!("Epoch change ledger info without a new validator set")
                    })?;

                let new_state = TrustedState {
                    latest_version: res_version,
                    verifier,
                };

                TrustedStateChange::Epoch {
                    new_state,
                    latest_li,
                    latest_epoch_change_li,
                    latest_validator_set,
                }
            }
            // We're still in the same epoch, just ratchet the version.
            None => {
                let latest_li = &res.ledger_info_with_sigs;

                let new_state = TrustedState {
                    latest_version: res_version,
                    verifier: self.verifier.clone(),
                };

                TrustedStateChange::Version {
                    new_state,
                    latest_li,
                }
            }
        };

        Ok(trusted_state_change)
    }

    pub fn latest_version(&self) -> Version {
        self.latest_version
    }

    pub fn verifier(&self) -> &VerifierType {
        &self.verifier
    }
}
