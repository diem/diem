// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::common::Round;
use libra_types::waypoint::Waypoint;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Public representation of the internal state of SafetyRules for monitoring / debugging purposes.
/// This does not include sensitive data like private keys.
/// @TODO add hash of ledger info (waypoint)
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConsensusState {
    epoch: u64,
    last_voted_round: Round,
    preferred_round: Round,
    waypoint: Waypoint,
}

impl Display for ConsensusState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ConsensusState: [\n\
             \tepoch = {},
             \tlast_voted_round = {},\n\
             \tpreferred_round = {}\n\
             \twaypoint = {}\n\
             ]",
            self.epoch, self.last_voted_round, self.preferred_round, self.waypoint,
        )
    }
}

impl ConsensusState {
    pub fn new(
        epoch: u64,
        last_voted_round: Round,
        preferred_round: Round,
        waypoint: Waypoint,
    ) -> Self {
        Self {
            epoch,
            last_voted_round,
            preferred_round,
            waypoint,
        }
    }

    /// Returns the current epoch
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns the last round that was voted on
    pub fn last_voted_round(&self) -> Round {
        self.last_voted_round
    }

    /// A "preferred block" is the two-chain head with the highest block round. The expectation is
    /// that a new proposal's parent is higher or equal to the preferred_round.
    pub fn preferred_round(&self) -> Round {
        self.preferred_round
    }

    /// Last known checkpoint this should map to a LedgerInfo that contains a new ValidatorSet
    pub fn waypoint(&self) -> Waypoint {
        self.waypoint
    }
}
