// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{
    block::Block,
    common::{Author, Round},
};

/// ProposerElection incorporates the logic of choosing a leader among multiple candidates.
/// We are open to a possibility for having multiple proposers per round, the ultimate choice
/// of a proposal is exposed by the election protocol via the stream of proposals.
pub trait ProposerElection<T> {
    /// If a given author is a valid candidate for being a proposer, generate the info,
    /// otherwise return None.
    /// Note that this function is synchronous.
    fn is_valid_proposer(&self, author: Author, round: Round) -> Option<Author>;

    /// Return all the possible valid proposers for a given round (this information can be
    /// used by e.g., voters for choosing the destinations for sending their votes to).
    fn get_valid_proposers(&self, round: Round) -> Vec<Author>;

    /// Notify proposer election about a new proposal. The function doesn't return any information:
    /// proposer election is going to notify the client about the chosen proposal via a dedicated
    /// channel (to be passed in constructor).
    fn process_proposal(&mut self, proposal: Block<T>) -> Option<Block<T>>;

    /// Take the highest ranked backup proposal if available for a given round
    /// (removes it from the struct),
    /// or returns None if no proposals have been received for a given round.
    /// A backup proposal is a valid proposal that was not chosen immediately in the
    /// `process_proposal()` return value.
    ///
    /// Note that once the backup proposal is taken and no other proposals are submitted, the
    /// following take requests are going to return None.
    fn take_backup_proposal(&mut self, round: Round) -> Option<Block<T>>;
}

// next continuously mutates a state and returns a u64-index
pub(crate) fn next(state: &mut Vec<u8>) -> u64 {
    // state = SHA-3-256(state)
    std::mem::replace(
        state,
        libra_crypto::HashValue::from_sha3_256(state).to_vec(),
    );
    let mut temp = [0u8; 8];
    temp.copy_from_slice(&state[..8]);
    // return state[0..8]
    u64::from_le_bytes(temp)
}
