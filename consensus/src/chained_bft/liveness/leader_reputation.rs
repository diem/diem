// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::liveness::proposer_election::{next, ProposerElection};
use consensus_types::{
    block::Block,
    common::{Author, Round},
};
use libra_types::block_metadata::BlockMetadata;
use serde::export::PhantomData;
use std::{cmp::Ordering, collections::HashSet, sync::Arc};

/// Interface to query committed BlockMetadata.
pub trait MetadataBackend {
    /// Return a window_size contiguous BlockMetadata window in which last one is at target_round or
    /// latest committed, return all previous one if not enough.
    fn get_block_metadata(&self, window_size: u64, target_round: Round) -> &[BlockMetadata];
}

/// Interface to calculate weights for proposers based on history.
pub trait ReputationHeuristic {
    /// Return the weights of all candidates based on the history.
    fn get_weights(&self, candidates: &[Author], history: &[BlockMetadata]) -> Vec<u64>;
}

/// If candidate appear in the history, it's assigned active_weight otherwise inactive weight.
pub struct ActiveInactiveHeuristic {
    active_weight: u64,
    inactive_weight: u64,
}

#[allow(dead_code)]
impl ActiveInactiveHeuristic {
    pub fn new(active_weight: u64, inactive_weight: u64) -> Self {
        Self {
            active_weight,
            inactive_weight,
        }
    }
}

impl ReputationHeuristic for ActiveInactiveHeuristic {
    fn get_weights(&self, candidates: &[Author], history: &[BlockMetadata]) -> Vec<u64> {
        let set = history.iter().fold(HashSet::new(), |mut set, meta| {
            set.insert(meta.proposer());
            set.extend(meta.voters().into_iter());
            set
        });
        candidates
            .iter()
            .map(|author| {
                if set.contains(&author) {
                    self.active_weight
                } else {
                    self.inactive_weight
                }
            })
            .collect()
    }
}

/// Committed history based proposer election implementation that could help bias towards
/// successful leaders to help improve performance.
pub struct LeaderReputation<T> {
    proposers: Vec<Author>,
    backend: Arc<dyn MetadataBackend>,
    window_size: u64,
    heuristic: Box<dyn ReputationHeuristic>,
    phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl<T> LeaderReputation<T> {
    pub fn new(
        proposers: Vec<Author>,
        backend: Arc<dyn MetadataBackend>,
        window_size: u64,
        heuristic: Box<dyn ReputationHeuristic>,
    ) -> Self {
        Self {
            proposers,
            backend,
            window_size,
            heuristic,
            phantom: PhantomData,
        }
    }
}

impl<T> ProposerElection<T> for LeaderReputation<T> {
    fn is_valid_proposer(&self, author: Author, round: Round) -> Option<Author> {
        if self.get_valid_proposers(round).contains(&author) {
            Some(author)
        } else {
            None
        }
    }

    fn get_valid_proposers(&self, round: Round) -> Vec<Author> {
        // TODO: configure the round gap
        let sliding_window = self.backend.get_block_metadata(self.window_size, round - 4);
        let mut weights = self.heuristic.get_weights(&self.proposers, sliding_window);
        assert_eq!(weights.len(), self.proposers.len());
        let mut total_weight = 0;
        for w in &mut weights {
            total_weight += *w;
            *w = total_weight;
        }
        let mut state = round.to_le_bytes().to_vec();
        let chosen_weight = next(&mut state) % total_weight;
        let chosen_index = weights
            .binary_search_by(|w| {
                if *w <= chosen_weight {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err();
        vec![self.proposers[chosen_index]]
    }

    fn process_proposal(&mut self, proposal: Block<T>) -> Option<Block<T>> {
        let author = proposal.author()?;
        if self.get_valid_proposers(proposal.round()).contains(&author) {
            Some(proposal)
        } else {
            None
        }
    }

    fn take_backup_proposal(&mut self, _round: Round) -> Option<Block<T>> {
        None
    }
}
