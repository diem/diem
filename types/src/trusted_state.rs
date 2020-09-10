// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    epoch_change::{EpochChangeProof, Verifier},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Version,
    waypoint::Waypoint,
};
use anyhow::{ensure, format_err, Result};
use std::{convert::TryFrom, sync::Arc};

/// `TrustedState` keeps track of our latest trusted state, including the latest
/// verified version and the latest verified validator set.
#[derive(Clone, Debug)]
pub struct TrustedState {
    /// The latest verified state is from either a waypoint or a ledger info, either
    /// inside an epoch or the epoch change ledger info.
    verified_state: Waypoint,
    /// The current verifier. If we're starting up fresh, this is probably a
    /// waypoint from our config. Otherwise, this is generated from the validator
    /// set in the last known epoch change ledger info.
    verifier: Arc<dyn Verifier>,
}

/// `TrustedStateChange` is the result of attempting to ratchet to a new trusted
/// state. In order to reduce redundant error checking, `TrustedStateChange` also
/// contains references to relevant items used to ratchet us.
#[derive(Clone, Debug)]
pub enum TrustedStateChange<'a> {
    /// We have a newer `TrustedState` but it's still in the same epoch, so only
    /// the latest trusted version changed.
    Version { new_state: TrustedState },
    /// We have a newer `TrustedState` and there was at least one epoch change,
    /// so we have a newer trusted version and a newer trusted validator set.
    Epoch {
        new_state: TrustedState,
        latest_epoch_change_li: &'a LedgerInfoWithSignatures,
    },
    /// The latest ledger info is at the same version as the trusted state and matches the hash.
    NoChange,
}

impl TrustedState {
    /// Verify and ratchet forward our trusted state using a `EpochChangeProof`
    /// (that moves us into the latest epoch) and a `LedgerInfoWithSignatures`
    /// inside that epoch.
    ///
    /// For example, a client sends an `UpdateToLatestLedgerRequest` to a
    /// FullNode and receives some epoch change proof along with a latest
    /// ledger info inside the `UpdateToLatestLedgerResponse`. This function
    /// verifies the change proof and ratchets the trusted state version forward
    /// if the response successfully moves us into a new epoch or a new latest
    /// ledger info within our current epoch.
    ///
    /// + If there was a validation error, e.g., the epoch change proof was
    /// invalid, we return an `Err`.
    ///
    /// + If the message was well formed but stale (i.e., the returned latest
    /// ledger is behind our trusted version), we also return an `Err` since
    /// stale responses should always be rejected.
    ///
    /// + If the response is fresh and there is no epoch change, we just ratchet
    /// our trusted version to the latest ledger info and return
    /// `Ok(TrustedStateChange::Version { .. })`.
    ///
    /// + If there is a new epoch and the server provides a correct proof, we
    /// ratchet our trusted version forward, update our verifier to contain
    /// the new validator set, and return `Ok(TrustedStateChange::Epoch { .. })`.
    pub fn verify_and_ratchet<'a>(
        &self,
        latest_li: &'a LedgerInfoWithSignatures,
        epoch_change_proof: &'a EpochChangeProof,
    ) -> Result<TrustedStateChange<'a>> {
        let res_version = latest_li.ledger_info().version();
        ensure!(
            res_version >= self.latest_version(),
            "The target latest ledger info is stale and behind our current trusted version",
        );

        if self
            .verifier
            .epoch_change_verification_required(latest_li.ledger_info().next_block_epoch())
        {
            // Verify the EpochChangeProof to move us into the latest epoch.
            let epoch_change_li = epoch_change_proof.verify(self.verifier.as_ref())?;
            let new_epoch_state = epoch_change_li
                .ledger_info()
                .next_epoch_state()
                .cloned()
                .ok_or_else(|| {
                    format_err!(
                        "A valid EpochChangeProof will never return a non-epoch change ledger info"
                    )
                })?;

            // Verify the latest ledger info inside the latest epoch.
            let new_verifier = Arc::new(new_epoch_state);

            // If these are the same, then we do not have a LI for the next Epoch and hence there
            // is nothing to verify.
            if epoch_change_li != latest_li {
                new_verifier.verify(latest_li)?;
            }

            let new_state = TrustedState {
                verified_state: Waypoint::new_any(latest_li.ledger_info()),
                verifier: new_verifier,
            };

            Ok(TrustedStateChange::Epoch {
                new_state,
                latest_epoch_change_li: epoch_change_li,
            })
        } else {
            // The EpochChangeProof is empty, stale, or only gets us into our
            // current epoch. We then try to verify that the latest ledger info
            // is this epoch.
            let new_waypoint = Waypoint::new_any(latest_li.ledger_info());
            if new_waypoint.version() == self.verified_state.version() {
                ensure!(
                    new_waypoint == self.verified_state,
                    "LedgerInfo doesn't match verified state"
                );
                Ok(TrustedStateChange::NoChange)
            } else {
                self.verifier.verify(latest_li)?;

                let new_state = TrustedState {
                    verified_state: new_waypoint,
                    verifier: self.verifier.clone(),
                };

                Ok(TrustedStateChange::Version { new_state })
            }
        }
    }

    pub fn latest_version(&self) -> Version {
        self.verified_state.version()
    }
}

impl From<Waypoint> for TrustedState {
    fn from(waypoint: Waypoint) -> Self {
        Self {
            verified_state: waypoint,
            verifier: Arc::new(waypoint),
        }
    }
}

impl TryFrom<&LedgerInfo> for TrustedState {
    type Error = anyhow::Error;

    fn try_from(ledger_info: &LedgerInfo) -> Result<Self> {
        let epoch_state = ledger_info.next_epoch_state().cloned().ok_or_else(|| {
            format_err!("No EpochState in LedgerInfo; it must not be on an epoch boundary")
        })?;

        Ok(Self {
            verified_state: Waypoint::new_epoch_boundary(ledger_info)?,
            verifier: Arc::new(epoch_state),
        })
    }
}
