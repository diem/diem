// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    epoch_change::{EpochChangeProof, Verifier},
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Version,
    waypoint::Waypoint,
};
use anyhow::{bail, ensure, format_err, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// `TrustedState` keeps track of our latest trusted state, including the latest
/// verified version and the latest verified validator set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TrustedState {
    /// The latest verified state is a waypoint either from a ledger info inside
    /// an epoch or an epoch change ledger info.
    verified_state: Waypoint,
    /// The current verifier. If we're starting up fresh, this is probably a
    /// waypoint from our config. Otherwise, this is generated from the validator
    /// set in the last known epoch change ledger info.
    verifier: TrustedStateVerifier,
}

/// The different epoch change [`Verifier`]s represented as an enum so we can
/// easily serialize the parent [`TrustedState`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum TrustedStateVerifier {
    EpochWaypoint(Waypoint),
    EpochState(EpochState),
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
    pub fn version(&self) -> Version {
        self.verified_state.version()
    }

    pub fn waypoint(&self) -> Waypoint {
        self.verified_state
    }

    /// Verify and ratchet forward our trusted state using a `EpochChangeProof`
    /// (that moves us into the latest epoch) and a `LedgerInfoWithSignatures`
    /// inside that epoch.
    ///
    /// For example, a client sends a `GetStateProof` request to an upstream
    /// FullNode and receives some epoch change proof along with a latest
    /// ledger info inside the `StateProof` response. This function
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
            res_version >= self.version(),
            "The target latest ledger info is stale and behind our current trusted version",
        );

        if self
            .verifier
            .epoch_change_verification_required(latest_li.ledger_info().next_block_epoch())
        {
            // Verify the EpochChangeProof to move us into the latest epoch.
            let epoch_change_li = epoch_change_proof.verify(&self.verifier)?;
            let new_epoch_state = epoch_change_li
                .ledger_info()
                .next_epoch_state()
                .cloned()
                .ok_or_else(|| {
                    format_err!(
                        "A valid EpochChangeProof will never return a non-epoch change ledger info"
                    )
                })?;

            // If the latest ledger info is in the same epoch as the new verifier, verify it and
            // use it as latest state, otherwise fallback to the epoch change ledger info.
            let new_epoch = new_epoch_state.epoch;
            let new_verifier = TrustedStateVerifier::EpochState(new_epoch_state);

            let verified_ledger_info = if epoch_change_li == latest_li {
                latest_li
            } else if latest_li.ledger_info().epoch() == new_epoch {
                new_verifier.verify(latest_li)?;
                latest_li
            } else if latest_li.ledger_info().epoch() > new_epoch && epoch_change_proof.more {
                epoch_change_li
            } else {
                bail!("Inconsistent epoch change proof and latest ledger info");
            };
            let verified_state = Waypoint::new_any(verified_ledger_info.ledger_info());

            let new_state = TrustedState {
                verified_state,
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
}

impl From<Waypoint> for TrustedState {
    fn from(waypoint: Waypoint) -> Self {
        Self {
            verified_state: waypoint,
            verifier: TrustedStateVerifier::EpochWaypoint(waypoint),
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
            verifier: TrustedStateVerifier::EpochState(epoch_state),
        })
    }
}

impl Verifier for TrustedStateVerifier {
    fn verify(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<()> {
        match self {
            Self::EpochWaypoint(inner) => Verifier::verify(inner, ledger_info),
            Self::EpochState(inner) => Verifier::verify(inner, ledger_info),
        }
    }

    fn epoch_change_verification_required(&self, epoch: u64) -> bool {
        match self {
            Self::EpochWaypoint(inner) => {
                Verifier::epoch_change_verification_required(inner, epoch)
            }
            Self::EpochState(inner) => Verifier::epoch_change_verification_required(inner, epoch),
        }
    }

    fn is_ledger_info_stale(&self, ledger_info: &LedgerInfo) -> bool {
        match self {
            Self::EpochWaypoint(inner) => Verifier::is_ledger_info_stale(inner, ledger_info),
            Self::EpochState(inner) => Verifier::is_ledger_info_stale(inner, ledger_info),
        }
    }
}

impl<'a> TrustedStateChange<'a> {
    pub fn new_state(self) -> Option<TrustedState> {
        match self {
            Self::Version { new_state } | Self::Epoch { new_state, .. } => Some(new_state),
            Self::NoChange => None,
        }
    }
}
