// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::liveness::proposer_election::ProposerElection;
use consensus_types::{
    block::Block,
    common::{Author, Payload, Round},
};
use libra_logger::prelude::*;

// next continuously mutates a state and returns a u64-index
pub fn next(state: &mut Vec<u8>) -> u64 {
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

/// The MultiProposer maps a round to an ordered list of authors.
/// The primary proposer is determined by an index of hash(round) % num_proposers.
/// The secondary proposer is determined by hash(hash(round)) % (num_proposers - 1), etc.
/// In order to ensure the required number of proposers, a set of the proposers to choose from
/// is updated after each hash: a chosen candidate is removed to avoid duplication.
///
/// Note the hash doesn't have to be cryptographic. The goal is to make sure that different
/// combinations of consecutive leaders are going to appear with equal probability.

/// While each round has more than a single valid proposer, only the primary proposer is
/// considered for `process_proposal`. The best backup proposer is returned in
/// `get_best_backup_proposal()`.
pub struct MultiProposer<T> {
    // Epoch is used as seed for electing different leaders per-epoch
    epoch: u64,
    // Ordering of proposers to rotate through (all honest replicas must agree on this)
    proposers: Vec<Author>,
    // Number of proposers per round
    num_proposers_per_round: usize,
    // Keeps the highest received backup (non-primary) proposal of the highest round.
    // When a proposal from a higher round is received, the previous proposals are discarded:
    // `process_proposal()` is supposed to be called only if the proposal round matches the current
    // round in the system, hence, proposals from previous rounds can be discarded.
    backup_proposal_round: Round,
    // The proposal is kept in a tuple (rank, block)
    backup_proposal: Option<(usize, Block<T>)>,
}

impl<T> MultiProposer<T> {
    pub fn new(epoch: u64, proposers: Vec<Author>, mut num_proposers_per_round: usize) -> Self {
        assert!(num_proposers_per_round > 0);
        if num_proposers_per_round > proposers.len() {
            error!(
                "num_proposers_per_round = {}, while there are just {} proposers, adjusting to {}",
                num_proposers_per_round,
                proposers.len(),
                proposers.len()
            );
            num_proposers_per_round = proposers.len();
        }

        assert!(num_proposers_per_round <= proposers.len());
        Self {
            epoch,
            proposers,
            num_proposers_per_round,
            backup_proposal_round: 0,
            backup_proposal: None,
        }
    }

    fn get_candidates(&self, round: Round) -> Vec<Author> {
        let mut res = vec![];
        let mut candidates = self.proposers.clone();
        // state = epoch | round
        let mut state = self.epoch.to_le_bytes().to_vec();
        state.extend_from_slice(&round.to_le_bytes());
        for _ in 0..self.num_proposers_per_round {
            let idx = next(&mut state);
            // note: this modular reduction has a slight bias.
            // Yet, the bias is so small in practice that it is not worth
            // addressing this edge case.
            let idx = idx % (candidates.len() as u64);
            res.push(candidates.swap_remove(idx as usize));
        }
        res
    }
}

impl<T: Payload> ProposerElection<T> for MultiProposer<T> {
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

    fn process_proposal(&mut self, proposal: Block<T>) -> Option<Block<T>> {
        let author = proposal.author()?;
        let round = proposal.round();
        let candidates = self.get_candidates(round);
        for (rank, candidate) in candidates.iter().enumerate() {
            if rank == 0 && author == *candidate {
                debug!(
                    "Primary proposal {}: going to process it right now.",
                    proposal
                );
                return Some(proposal);
            }
            if author == *candidate {
                // This is a valid non-primary proposal, add it to backup_proposals.
                debug!(
                    "Secondary proposal {}: will process it if no primary available.",
                    proposal
                );
                if round > self.backup_proposal_round {
                    self.backup_proposal = Some((rank, proposal));
                    self.backup_proposal_round = round;
                } else if round == self.backup_proposal_round {
                    // Already have some backup for the given round: choose the best (lowest) rank.
                    let current_rank = self
                        .backup_proposal
                        .as_ref()
                        .map_or(std::usize::MAX, |(r, _)| *r);
                    if rank < current_rank {
                        self.backup_proposal = Some((rank, proposal));
                    }
                }
                return None;
            }
        }
        warn!(
            "Proposal {} does not match any candidate for round {}, ignore.",
            proposal, round
        );

        None
    }

    fn take_backup_proposal(&mut self, round: Round) -> Option<Block<T>> {
        if self.backup_proposal_round != round {
            return None;
        }
        if let Some((_, block)) = self.backup_proposal.take() {
            return Some(block);
        }

        None
    }
}
