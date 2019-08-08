// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{common::Round, consensus_types::proposal_info::ProposalInfo};

/// ProposerElection incorporates the logic of choosing a leader among multiple candidates.
/// We are open to a possibility for having multiple proposers per round, the ultimate choice
/// of a proposal is exposed by the election protocol via the stream of proposals.
pub trait ProposerElection<T, P> {
    /// If a given author is a valid candidate for being a proposer, generate the info,
    /// otherwise return None.
    /// Note that this function is synchronous.
    fn is_valid_proposer(&self, author: P, round: Round) -> Option<P>;

    /// Return all the possible valid proposers for a given round (this information can be
    /// used by e.g., voters for choosing the destinations for sending their votes to).
    fn get_valid_proposers(&self, round: Round) -> Vec<P>;

    /// Notify proposer election about a new proposal. The function doesn't return any information:
    /// proposer election is going to notify the client about the chosen proposal via a dedicated
    /// channel (to be passed in constructor).
    fn process_proposal(&self, proposal: ProposalInfo<T, P>) -> Option<ProposalInfo<T, P>>;
}
