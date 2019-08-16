// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Payload, Round},
    consensus_types::block::Block,
    liveness::proposer_election::ProposerElection,
};

use siphasher::sip::SipHasher24;
use std::hash::{Hash, Hasher};

/// The MultiProposer maps a round to an ordered list of authors.
/// The primary proposer is determined by an index of hash(round) % num_proposers.
/// The secondary proposer is determined by hash(hash(round)) % (num_proposers - 1), etc.
/// In order to ensure the required number of proposers, a set of the proposers to choose from
/// is updated after each hash: a chosen candidate is removed to avoid duplication.
///
/// Note the hash doesn't have to be cryptographic. The goal is to make sure that different
/// combinations of consecutive leaders are going to appear with equal probability.

/// While each round has more than a single valid proposer, only the primary proposer is
/// considered for `process_proposal`.
/// TODO: we're going to extend this functionality with an ability to use secondary proposals upon
/// timeout.
pub struct MultiProposer {
    // Ordering of proposers to rotate through (all honest replicas must agree on this)
    proposers: Vec<Author>,
    // Number of proposers per round
    num_proposers_per_round: usize,
}

impl MultiProposer {
    pub fn new(proposers: Vec<Author>, num_proposers_per_round: usize) -> Self {
        assert!(num_proposers_per_round > 0);
        assert!(num_proposers_per_round <= proposers.len());
        Self {
            proposers,
            num_proposers_per_round,
        }
    }

    // A deterministic hashing function based on SipHash 2-4 hasher
    pub fn hash(val: u64) -> u64 {
        let mut hasher = SipHasher24::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    fn get_candidates(&self, round: Round) -> Vec<Author> {
        let mut res = vec![];
        let mut candidates = self.proposers.clone();
        let mut cur_val = round;
        for _ in 0..self.num_proposers_per_round {
            cur_val = Self::hash(cur_val);
            let idx = (cur_val % candidates.len() as u64) as usize;
            res.push(candidates.swap_remove(idx));
        }
        res
    }
}

impl<T: Payload> ProposerElection<T> for MultiProposer {
    fn is_valid_proposer(&self, author: Author, round: Round) -> Option<Author> {
        if self.get_candidates(round).contains(&author) {
            Some(author)
        } else {
            None
        }
    }

    fn get_valid_proposers(&self, round: Round) -> Vec<Author> {
        self.get_candidates(round)
    }

    fn process_proposal(&self, proposal: Block<T>) -> Option<Block<T>> {
        // At this phase we're considering the primary proposer only.
        if let Some(author) = proposal.author() {
            if author == self.get_candidates(proposal.round())[0] {
                return Some(proposal);
            }
        }
        None
    }
}
