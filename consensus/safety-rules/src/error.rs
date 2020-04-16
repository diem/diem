// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::common::Round;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Error, PartialEq, Serialize)]
/// Different reasons for proposal rejection
pub enum Error {
    #[error("Timeout round, {0}, is incompatible with last votedx round, {1}")]
    BadTimeoutLastVotedRound(u64, u64),

    #[error("Timeout round, {0}, is incompatible with preferred round, {1}")]
    BadTimeoutPreferredRound(u64, u64),

    #[error("Provided epoch, {0}, does not match expected epoch, {1}")]
    IncorrectEpoch(u64, u64),

    #[error("Internal error: {:?}", error)]
    InternalError { error: String },

    #[error("Unable to verify that the new tree extneds the parent: {:?}", error)]
    InvalidAccumulatorExtension { error: String },

    #[error("No next_validator_set specified in the provided Ledger Info")]
    InvalidLedgerInfo,

    /// This proposal's round is less than round of preferred block.
    /// Returns the id of the preferred block.
    #[error(
        "Proposal's round is lower than round of preferred block at round {:?}",
        preferred_round
    )]
    ProposalRoundLowerThenPreferredBlock { preferred_round: Round },

    /// This proposal is too old - return last_voted_round
    #[error(
        "Proposal at round {:?} is not newer than the last vote round {:?}",
        proposal_round,
        last_voted_round
    )]
    OldProposal {
        last_voted_round: Round,
        proposal_round: Round,
    },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Waypoint mismatch: {0}")]
    WaypointMismatch(String),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalError {
            error: format!("{}", error),
        }
    }
}

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<libra_secure_net::Error> for Error {
    fn from(error: libra_secure_net::Error) -> Self {
        Self::InternalError {
            error: format!("{}", error),
        }
    }
}
