// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow;
use consensus_types::common::Round;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
/// Different reasons for proposal rejection
pub enum Error {
    #[error("Internal error: {:?}", error)]
    InternalError { error: String },

    #[error("Unable to verify that the new tree extneds the parent: {:?}", error)]
    InvalidAccumulatorExtension { error: String },

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
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalError {
            error: format!("{}", error),
        }
    }
}
