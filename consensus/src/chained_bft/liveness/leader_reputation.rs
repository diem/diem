// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::liveness::proposer_election::{next, ProposerElection};
use consensus_types::{
    block::Block,
    common::{Author, Round},
};
use libra_types::block_metadata::{new_block_event_key, NewBlockEvent};
use libradb::LibraDBTrait;
use serde::export::PhantomData;
use std::{
    cmp::Ordering,
    collections::HashSet,
    sync::{Arc, Mutex},
};

/// Interface to query committed BlockMetadata.
pub trait MetadataBackend: Send + Sync {
    /// Return a contiguous BlockMetadata window in which last one is at target_round or
    /// latest committed, return all previous one if not enough.
    fn get_block_metadata(&self, target_round: Round) -> Vec<NewBlockEvent>;
}

pub struct LibraDBBackend {
    window_size: usize,
    libra_db: Arc<dyn LibraDBTrait>,
    window: Mutex<Vec<(u64, NewBlockEvent)>>,
}

impl LibraDBBackend {
    pub fn new(window_size: usize, libra_db: Arc<dyn LibraDBTrait>) -> Self {
        Self {
            window_size,
            libra_db,
            window: Mutex::new(vec![]),
        }
    }

    fn refresh_window(&self, target_round: Round) {
        // assumes target round is not too far from latest commit
        let buffer = 10;
        let events = self
            .libra_db
            .get_events(
                &new_block_event_key(),
                u64::max_value(),
                self.window_size as u64 + buffer,
            )
            .unwrap();
        let events: Vec<_> = events
            .into_iter()
            .map(|(v, e)| (v, lcs::from_bytes::<NewBlockEvent>(e.event_data()).unwrap()))
            .filter(|(_, e)| e.round() <= target_round)
            .take(self.window_size)
            .collect();
        *self.window.lock().unwrap() = events;
    }
}

impl MetadataBackend for LibraDBBackend {
    // assume the target_round only increases
    fn get_block_metadata(&self, target_round: Round) -> Vec<NewBlockEvent> {
        let (known_version, known_round) = self
            .window
            .lock()
            .unwrap()
            .first()
            .map(|(v, e)| (*v, e.round()))
            .unwrap_or((0, 0));
        if !(known_round == target_round
            || known_version == self.libra_db.get_latest_version().unwrap())
        {
            self.refresh_window(target_round);
        }
        self.window
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .map(|(_, e)| e)
            .collect()
    }
}

/// Interface to calculate weights for proposers based on history.
pub trait ReputationHeuristic: Send + Sync {
    /// Return the weights of all candidates based on the history.
    fn get_weights(&self, candidates: &[Author], history: &[NewBlockEvent]) -> Vec<u64>;
}

/// If candidate appear in the history, it's assigned active_weight otherwise inactive weight.
pub struct ActiveInactiveHeuristic {
    active_weight: u64,
    inactive_weight: u64,
}

impl ActiveInactiveHeuristic {
    pub fn new(active_weight: u64, inactive_weight: u64) -> Self {
        Self {
            active_weight,
            inactive_weight,
        }
    }
}

impl ReputationHeuristic for ActiveInactiveHeuristic {
    fn get_weights(&self, candidates: &[Author], history: &[NewBlockEvent]) -> Vec<u64> {
        let set = history.iter().fold(HashSet::new(), |mut set, meta| {
            set.insert(meta.proposer());
            set.extend(meta.votes().into_iter());
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
    backend: Box<dyn MetadataBackend>,
    heuristic: Box<dyn ReputationHeuristic>,
    phantom: PhantomData<T>,
}

impl<T> LeaderReputation<T> {
    pub fn new(
        proposers: Vec<Author>,
        backend: Box<dyn MetadataBackend>,
        heuristic: Box<dyn ReputationHeuristic>,
    ) -> Self {
        Self {
            proposers,
            backend,
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
        let target_round = if round >= 4 { round - 4 } else { 0 };
        let sliding_window = self.backend.get_block_metadata(target_round);
        let mut weights = self.heuristic.get_weights(&self.proposers, &sliding_window);
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
