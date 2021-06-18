// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{common::Round, safety_data::SafetyData};
use diem_types::waypoint::Waypoint;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Public representation of the internal state of SafetyRules for monitoring / debugging purposes.
/// This does not include sensitive data like private keys.
/// @TODO add hash of ledger info (waypoint)
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConsensusState {
    safety_data: SafetyData,
    waypoint: Waypoint,
    in_validator_set: bool,
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
             \tin_validator_set = {}\n\
             ]",
            self.epoch(),
            self.last_voted_round(),
            self.preferred_round(),
            self.waypoint,
            self.in_validator_set,
        )
    }
}

impl ConsensusState {
    pub fn new(safety_data: SafetyData, waypoint: Waypoint, in_validator_set: bool) -> Self {
        Self {
            safety_data,
            waypoint,
            in_validator_set,
        }
    }

    /// Returns the current epoch
    pub fn epoch(&self) -> u64 {
        self.safety_data.epoch
    }

    /// Returns the last round that was voted on
    pub fn last_voted_round(&self) -> Round {
        self.safety_data.last_voted_round
    }

    /// A "preferred block" is the two-chain head with the highest block round. The expectation is
    /// that a new proposal's parent is higher or equal to the preferred_round.
    pub fn preferred_round(&self) -> Round {
        self.safety_data.preferred_round
    }

    /// The round of the highest QuorumCert.
    pub fn one_chain_round(&self) -> Round {
        self.safety_data.one_chain_round
    }

    /// Last known checkpoint this should map to a LedgerInfo that contains a new ValidatorSet
    pub fn waypoint(&self) -> Waypoint {
        self.waypoint
    }

    /// Indicating whether the validator is validator set
    pub fn in_validator_set(&self) -> bool {
        self.in_validator_set
    }

    /// Return a copy of the safety data.
    pub fn safety_data(&mut self) -> SafetyData {
        self.safety_data.clone()
    }
}
