// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::liveness::proposer_election::ProposerElection;
use consensus_types::{
    block::Block,
    common::{Author, Payload, Round},
};

/// The rotating proposer maps a round to an author according to a round-robin rotation.
/// A fixed proposer strategy loses liveness when the fixed proposer is down. Rotating proposers
/// won't gather quorum certificates to machine loss/byzantine behavior on f/n rounds.
pub struct RotatingProposer {
    // Ordering of proposers to rotate through (all honest replicas must agree on this)
    proposers: Vec<Author>,
    // Number of contiguous rounds (i.e. round numbers increase by 1) a proposer is active
    // in a row
    contiguous_rounds: u32,
}

/// Choose a proposer that is going to be the single leader (relevant for a mock fixed proposer
/// election only).
pub fn choose_leader(peers: Vec<Author>) -> Author {
    // As it is just a tmp hack function, pick the min PeerId to be a proposer.
    // TODO: VRF will be integrated later.
    peers.into_iter().min().expect("No trusted peers found!")
}

impl RotatingProposer {
    /// With only one proposer in the vector, it behaves the same as a fixed proposer strategy.
    pub fn new(proposers: Vec<Author>, contiguous_rounds: u32) -> Self {
        Self {
            proposers,
            contiguous_rounds,
        }
    }

    fn get_proposer(&self, round: Round) -> Author {
        self.proposers
            [((round / u64::from(self.contiguous_rounds)) % self.proposers.len() as u64) as usize]
    }
}

impl<T: Payload> ProposerElection<T> for RotatingProposer {
    fn is_valid_proposer(&self, author: Author, round: Round) -> Option<Author> {
        if self.get_proposer(round) == author {
            Some(author)
        } else {
            None
        }
    }

    fn get_valid_proposers(&self, round: Round) -> Vec<Author> {
        vec![self.get_proposer(round)]
    }

    fn process_proposal(&mut self, proposal: Block<T>) -> Option<Block<T>> {
        // This is a simple rotating proposer, the proposal is processed in the context of the
        // caller task, no synchronization required because there is no mutable state.
        let round_author = self.get_proposer(proposal.round());
        if Some(round_author) != proposal.author() {
            None
        } else {
            Some(proposal)
        }
    }

    fn take_backup_proposal(&mut self, _round: Round) -> Option<Block<T>> {
        None
    }
}
